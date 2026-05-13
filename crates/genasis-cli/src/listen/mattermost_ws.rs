//! Real Mattermost flavor — `/api/v4/websocket` 구독 + `/api/v4/posts`
//! POST + Plane REST 로 카드 transition. genesis §28 의 trial 등가물에
//! 대응하는 실 운영 경로.
//!
//! Mattermost WS 프로토콜:
//!   1. `wss://<host>/api/v4/websocket` 으로 connect
//!   2. 클라이언트가 `{"seq":1,"action":"authentication_challenge","data":{"token":"<bot_token>"}}` 전송
//!   3. 서버가 `event="hello"` 응답 → ready
//!   4. 이후 모든 frame 은 JSON. `event="posted"` 만 필터.
//!      `data.post` 는 JSON string 안에 또 JSON — 한 번 더 parse.
//!
//! 재연결: exponential backoff 1s → 2s → 4s → 8s → 16s → 30s 후 30s 고정.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{info, warn};


use super::{EventStream, InboundEvent};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub struct MattermostWsStream {
    ws_url: String,
    bot_token: String,
    socket: Option<WsStream>,
    seq: u64,
    /// 우리 측 agent bot user_id 집합 — `is_human` 판정에 사용. 비어
    /// 있으면 "역할 키워드 외 actor" 기준 fallback.
    agent_user_ids: Vec<String>,
}

impl MattermostWsStream {
    /// `mm_base_url` 은 http(s)://… 형태. WS endpoint 는 자동으로
    /// ws(s)://…/api/v4/websocket 으로 변환.
    pub async fn connect(
        mm_base_url: &str,
        bot_token: &str,
        agent_user_ids: Vec<String>,
    ) -> Result<Self> {
        let ws_url = derive_ws_url(mm_base_url);
        let mut s = Self {
            ws_url,
            bot_token: bot_token.to_string(),
            socket: None,
            seq: 1,
            agent_user_ids,
        };
        s.reconnect().await?;
        Ok(s)
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(30);
        loop {
            match connect_async(&self.ws_url).await {
                Ok((sock, _)) => {
                    self.socket = Some(sock);
                    self.seq = 1;
                    // auth challenge
                    let auth = json!({
                        "seq": self.seq,
                        "action": "authentication_challenge",
                        "data": {"token": self.bot_token},
                    });
                    self.seq += 1;
                    if let Some(s) = &mut self.socket {
                        s.send(WsMessage::Text(auth.to_string().into())).await?;
                    }
                    info!(target: "listen", ws_url = %self.ws_url, "Mattermost WS connected + auth sent");
                    return Ok(());
                }
                Err(e) => {
                    warn!("Mattermost WS connect failed: {e} — retry in {:?}", backoff);
                    sleep(backoff).await;
                    backoff = (backoff * 2).min(max_backoff);
                }
            }
        }
    }
}

fn derive_ws_url(http_url: &str) -> String {
    let base = http_url.trim_end_matches('/');
    if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}/api/v4/websocket")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}/api/v4/websocket")
    } else {
        format!("ws://{base}/api/v4/websocket")
    }
}

#[async_trait]
impl EventStream for MattermostWsStream {
    async fn next_event(&mut self) -> Result<InboundEvent> {
        loop {
            let sock = match self.socket.as_mut() {
                Some(s) => s,
                None => {
                    self.reconnect().await?;
                    continue;
                }
            };
            let frame = match sock.next().await {
                Some(Ok(WsMessage::Text(t))) => t,
                Some(Ok(WsMessage::Binary(_)))
                | Some(Ok(WsMessage::Ping(_)))
                | Some(Ok(WsMessage::Pong(_)))
                | Some(Ok(WsMessage::Frame(_))) => continue,
                Some(Ok(WsMessage::Close(_))) | None => {
                    warn!("Mattermost WS closed — reconnecting");
                    self.socket = None;
                    continue;
                }
                Some(Err(e)) => {
                    warn!("Mattermost WS error {e} — reconnecting");
                    self.socket = None;
                    continue;
                }
            };
            let payload: Value = match serde_json::from_str(&frame) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Mattermost WS non-JSON frame ({e}): {frame}");
                    continue;
                }
            };
            let event = payload.get("event").and_then(|x| x.as_str()).unwrap_or("");
            if event != "posted" {
                continue;
            }
            // data.post 는 JSON string 안에 또 JSON.
            let post_raw = payload
                .pointer("/data/post")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            let post: Value = match serde_json::from_str(post_raw) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Mattermost data.post parse error {e}: {post_raw}");
                    continue;
                }
            };
            let user_id = post
                .get("user_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let channel_id = post
                .get("channel_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let message = post
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let post_id = post
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let thread_root_id = post
                .get("root_id")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let channel_name = payload
                .pointer("/data/channel_name")
                .and_then(|x| x.as_str())
                .map(String::from);
            if channel_id.is_empty() || message.trim().is_empty() {
                continue;
            }
            // is_human: user_id 가 우리 agent bot 집합에 없으면 human.
            // agent_user_ids 가 비어 있는 환경 (테스트) 에서는 actor name
            // 휴리스틱 fallback.
            let is_human = if self.agent_user_ids.is_empty() {
                super::is_human_actor(&user_id)
            } else {
                !self.agent_user_ids.iter().any(|id| id == &user_id)
            };
            return Ok(InboundEvent::PostCreated {
                post_id,
                channel_id,
                channel_name,
                actor: user_id,
                message,
                thread_root_id,
                is_human,
            });
        }
    }
}
