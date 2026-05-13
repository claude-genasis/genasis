//! Trial flavor — trial-app `/api/events/stream` SSE 구독 + `/api/mattermost/posts`
//! POST 로 응답 + `/api/trial/bootstrap` 으로 카드 transition.
//!
//! `reqwest-eventsource` v0.6 를 사용하여 chunked SSE 의 reconnect /
//! retry / heartbeat 를 안전하게 처리. 이전 사이클의 raw `bytes_stream`
//! timeout 문제는 그 crate 의 추상화로 해소.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event as SseEvent, EventSource};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, warn};

use super::{is_human_actor, EventStream, InboundEvent};

pub struct TrialAppSseStream {
    sse: EventSource,
    base_url: String,
    team_token: String,
    client: Client,
    /// D-024: 연속 reconnect 시도 횟수. exponential backoff 의 기준.
    consecutive_reconnects: u32,
}

impl TrialAppSseStream {
    pub fn new(base_url: &str, team_token: &str) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .build()?;
        let sse = Self::open(&client, base_url, team_token)?;
        Ok(Self {
            sse,
            base_url: base_url.trim_end_matches('/').to_string(),
            team_token: team_token.to_string(),
            client,
            consecutive_reconnects: 0,
        })
    }

    fn open(client: &Client, base_url: &str, team_token: &str) -> Result<EventSource> {
        let url = format!(
            "{}/api/events/stream?team={}",
            base_url.trim_end_matches('/'),
            team_token
        );
        let request = client
            .get(&url)
            .header("X-Genasis-Team-Token", team_token)
            .header("Accept", "text/event-stream");
        EventSource::new(request).map_err(|e| anyhow!("EventSource init failed: {e}"))
    }

    /// D-024: 죽은 EventSource 를 새 인스턴스로 swap. exponential backoff
    /// 1s → 2s → 4s → 8s → max 30s 로 부하 분산.
    async fn rebuild(&mut self) -> Result<()> {
        self.consecutive_reconnects = self.consecutive_reconnects.saturating_add(1);
        let backoff_secs = match self.consecutive_reconnects {
            1 => 1,
            2 => 2,
            3 => 4,
            4 => 8,
            5 => 16,
            _ => 30,
        };
        info!(
            target: "listen",
            attempt = self.consecutive_reconnects,
            backoff_secs = backoff_secs,
            "trial SSE rebuild — sleeping then opening new EventSource"
        );
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        self.sse = Self::open(&self.client, &self.base_url, &self.team_token)?;
        Ok(())
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn team_token(&self) -> &str {
        &self.team_token
    }
}

#[async_trait]
impl EventStream for TrialAppSseStream {
    async fn next_event(&mut self) -> Result<InboundEvent> {
        loop {
            // D-024: stream 이 None 을 반환하면 reqwest-eventsource 가
            // 영구 종료 상태 — `next()` 를 다시 호출해도 영원히 None.
            // 그 경우 새 EventSource 를 만들어 swap 후 다시 시도.
            let msg = match self.sse.next().await {
                Some(m) => m,
                None => {
                    warn!(
                        target: "listen",
                        "trial SSE returned None (stream permanently closed) — rebuilding"
                    );
                    self.rebuild().await?;
                    continue;
                }
            };
            match msg {
                Ok(SseEvent::Open) => {
                    info!(target: "listen", "trial SSE opened");
                    // 새 연결 성공시 backoff counter 리셋
                    self.consecutive_reconnects = 0;
                    continue;
                }
                Ok(SseEvent::Message(m)) => {
                    // trial-app SSE 포맷: `event: <kind>` 라인이 분류,
                    // `data: <json>` 가 곧 payload (래핑 없음). connected
                    // 같은 housekeeping 이벤트는 skip.
                    if m.event != "post.created" {
                        continue;
                    }
                    let p: Value = match serde_json::from_str(&m.data) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("trial SSE non-JSON data ({e}): {}", m.data);
                            continue;
                        }
                    };
                    let actor = p.get("actor").and_then(|x| x.as_str()).unwrap_or("");
                    let channel_id = p
                        .get("channel_id")
                        .and_then(|x| x.as_i64())
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let message = p
                        .get("message")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    if channel_id.is_empty() || message.trim().is_empty() {
                        continue;
                    }
                    let post_id = p
                        .get("id")
                        .and_then(|x| x.as_i64())
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let thread_root_id = p
                        .get("root_id")
                        .and_then(|x| x.as_i64())
                        .map(|i| i.to_string());
                    return Ok(InboundEvent::PostCreated {
                        team_token: self.team_token.clone(),
                        post_id,
                        channel_id,
                        channel_name: None,
                        actor: actor.to_string(),
                        message,
                        thread_root_id,
                        is_human: is_human_actor(actor),
                    });
                }
                Err(e) => {
                    // D-024: reqwest-eventsource 의 자동 retry 가 일부
                    // transport 에러를 처리하지만 영구 실패 시 다음 next()
                    // 가 None. 빠른 loop 보호용 sleep + 다음 turn 에서
                    // rebuild 결정.
                    warn!("trial SSE transport error: {e}");
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }
            }
        }
    }
}
