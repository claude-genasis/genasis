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

use super::{is_human_actor, message_requests_done, EventSink, EventStream, InboundEvent};

pub struct TrialAppSseStream {
    sse: EventSource,
    base_url: String,
    team_token: String,
}

impl TrialAppSseStream {
    pub fn new(base_url: &str, team_token: &str) -> Result<Self> {
        let url = format!(
            "{}/api/events/stream?team={}",
            base_url.trim_end_matches('/'),
            team_token
        );
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .build()?;
        let request = client
            .get(&url)
            .header("X-Genasis-Team-Token", team_token)
            .header("Accept", "text/event-stream");
        let sse = EventSource::new(request)
            .map_err(|e| anyhow!("EventSource init failed: {e}"))?;
        Ok(Self {
            sse,
            base_url: base_url.trim_end_matches('/').to_string(),
            team_token: team_token.to_string(),
        })
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
            let msg = self
                .sse
                .next()
                .await
                .ok_or_else(|| anyhow!("SSE stream closed unexpectedly"))?;
            match msg {
                Ok(SseEvent::Open) => {
                    info!(target: "listen", "trial SSE opened");
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
                    warn!("trial SSE transport error: {e} — eventsource will reconnect");
                    // reqwest-eventsource 의 default retry 정책이 자동 재시도. 명시
                    // sleep 은 불필요하나 loop 보호를 위해 5ms 정도.
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            }
        }
    }
}

pub struct TrialAppSink {
    base_url: String,
    team_token: String,
    project_slug: String,
    project_name: String,
    client: Client,
}

impl TrialAppSink {
    pub fn new(base_url: &str, team_token: &str, project_slug: &str, project_name: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            team_token: team_token.to_string(),
            project_slug: project_slug.to_string(),
            project_name: project_name.to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(20))
                .build()
                .expect("reqwest Client"),
        }
    }
}

#[async_trait]
impl EventSink for TrialAppSink {
    async fn reply(
        &self,
        triggered_by: &InboundEvent,
        actor: &str,
        text: &str,
    ) -> Result<()> {
        let (channel_id, root_id) = match triggered_by {
            InboundEvent::PostCreated {
                channel_id,
                thread_root_id,
                ..
            } => (channel_id.clone(), thread_root_id.clone()),
        };
        let cid: i64 = channel_id
            .parse()
            .map_err(|e| anyhow!("invalid sim channel_id {channel_id}: {e}"))?;
        let mut body = json!({
            "channel_id": cid,
            "actor": actor,
            "message": text,
        });
        if let Some(rid) = root_id {
            if let Ok(rid_i) = rid.parse::<i64>() {
                body["root_id"] = json!(rid_i);
            }
        }
        let url = format!("{}/api/mattermost/posts", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("X-Genasis-Team-Token", &self.team_token)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "trial reply POST {url} → {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            );
        }
        Ok(())
    }

    async fn maybe_transition_for_directive(&self, message: &str) -> Result<()> {
        if !message_requests_done(message) {
            return Ok(());
        }
        // 보수적 기본: init 4 카드 + publish 1 카드 모두 done 으로 재bootstrap.
        // ensureIssue idempotent + state-aware (agents-pool@8b03654) 라 안전.
        let titles = [
            "Set up agentic team (you are here)",
            "Write PRD and split into tickets",
            "Build the example app from PRD",
            "🎉 Example app published — open showcase",
        ];
        let body = json!({
            "team_token": self.team_token,
            "project": {"slug": self.project_slug, "name": self.project_name},
            "channels": [{
                "key": "scrum",
                "name": format!("scrum-{}", self.project_slug),
                "display_name": format!("{} — Scrum", self.project_name),
            }],
            "demo_issues": titles.iter().map(|t| json!({
                "title": t, "state": "done", "assignee": "genasis",
            })).collect::<Vec<_>>(),
        });
        let url = format!("{}/api/trial/bootstrap", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("X-Genasis-Team-Token", &self.team_token)
            .json(&body)
            .send()
            .await?;
        if resp.status().is_success() {
            info!(
                target: "listen",
                "trial: transitioned init cards → done via bootstrap"
            );
        } else {
            warn!(
                "trial transition bootstrap {url} → {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            );
        }
        Ok(())
    }
}
