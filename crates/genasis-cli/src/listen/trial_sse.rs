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

use super::routing::PmRouting;
use super::{is_human_actor, message_requests_done, EventSink, EventStream, InboundEvent};

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
    async fn reply(&self, triggered_by: &InboundEvent, actor: &str, text: &str) -> Result<()> {
        let (channel_id, root_id, source_post_id) = match triggered_by {
            InboundEvent::PostCreated {
                channel_id,
                thread_root_id,
                post_id,
                ..
            } => (channel_id.clone(), thread_root_id.clone(), post_id.clone()),
        };
        let cid: i64 = channel_id
            .parse()
            .map_err(|e| anyhow!("invalid sim channel_id {channel_id}: {e}"))?;
        // ADR-018 + strategy.md §9: 모든 agent 응답은 사람 요청의 스레드
        // 안에 reply 한다 (`root_id = 사람 post id`). 사람 메시지가 root
        // post 면 thread_root_id 가 null 이므로 그 자체 post_id 를 root
        // 로 사용. 이미 thread 안 메시지면 그 root_id 그대로 사용해서
        // 같은 thread 에 reply.
        let effective_root = root_id.or(Some(source_post_id));
        let mut body = json!({
            "channel_id": cid,
            "actor": actor,
            "message": text,
        });
        if let Some(rid) = effective_root {
            if let Ok(rid_i) = rid.parse::<i64>() {
                if rid_i > 0 {
                    body["root_id"] = json!(rid_i);
                }
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

    async fn apply_pm_routing(
        &self,
        routing: &PmRouting,
    ) -> Result<std::collections::HashMap<String, u64>> {
        let mut seq_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        // (a) app_kind / app_features 가 명시되면 sim_teams 업데이트
        if routing.app_kind.is_some() || !routing.app_features.is_empty() {
            let mut body = json!({
                "team_token": self.team_token,
                "status": "complete",
                "project": {"slug": self.project_slug, "name": self.project_name},
                "app_features": routing.app_features,
            });
            if let Some(k) = &routing.app_kind {
                body["app_kind"] = json!(k);
            }
            let url = format!("{}/api/trial/team-app/status", self.base_url);
            match self
                .client
                .post(&url)
                .header("X-Genasis-Team-Token", &self.team_token)
                .json(&body)
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    info!(
                        target: "listen",
                        kind = ?routing.app_kind,
                        features = ?routing.app_features,
                        "trial: sim_teams app_kind/features updated"
                    );
                }
                Ok(r) => warn!(
                    "trial team-app/status update {} → {}: {}",
                    url,
                    r.status(),
                    r.text().await.unwrap_or_default()
                ),
                Err(e) => warn!("trial team-app/status update {url}: {e}"),
            }
        }

        // (b) new_cards + transitions 를 bootstrap idempotent demo_issues
        // 로 보내서 sim_issues 에 INSERT + state 동기화. ensureIssue 가
        // (team_token, project_slug, title) 기준 dedup + state-aware
        // (agents-pool@8b03654) 이므로 신규 카드도 transition 도 같은
        // 한 round-trip 으로 해결.
        if !routing.new_cards.is_empty() || !routing.transitions.is_empty() {
            let mut demo_issues: Vec<Value> = Vec::new();
            for c in &routing.new_cards {
                demo_issues.push(json!({
                    "title": c.title,
                    "state": c.state,
                    "assignee": c.assignee.clone().unwrap_or_default(),
                }));
            }
            for t in &routing.transitions {
                // title_substring 이 정확한 제목 매칭은 아닐 수 있어 일단
                // 그대로 보내고 ensureIssue 가 title 기준 dedup → 있으면
                // state-sync. 없으면 새 row 가 생기되 placeholder.
                demo_issues.push(json!({
                    "title": t.title_substring,
                    "state": t.to_state,
                    "assignee": "agent",
                }));
            }
            let body = json!({
                "team_token": self.team_token,
                "project": {"slug": self.project_slug, "name": self.project_name},
                "channels": [{
                    "key": "scrum",
                    "name": format!("scrum-{}", self.project_slug),
                    "display_name": format!("{} — Scrum", self.project_name),
                }],
                "demo_issues": demo_issues,
            });
            let url = format!("{}/api/trial/bootstrap", self.base_url);
            match self
                .client
                .post(&url)
                .header("X-Genasis-Team-Token", &self.team_token)
                .json(&body)
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    info!(
                        target: "listen",
                        cards_in = routing.new_cards.len(),
                        transitions = routing.transitions.len(),
                        "trial: sim_issues seed/transition via bootstrap"
                    );
                    // D-037: 응답의 `demo_issues[]` 가 각 카드의 sequence_id
                    // 를 담고 있음. title → sequence_id 매핑으로 데몬 fan-out
                    // 이 agent prompt 의 `#N` placeholder 를 실제 카드 번호로
                    // 대체. Plane 호환: real Plane 의 sequence_id 와 같은
                    // 정수 형식이라 v0.6.0 에서도 같은 코드 흐름 가능.
                    if let Ok(body) = r.json::<Value>().await {
                        if let Some(arr) = body.get("demo_issues").and_then(|v| v.as_array()) {
                            for issue in arr {
                                if let (Some(title), Some(seq)) = (
                                    issue.get("title").and_then(|t| t.as_str()),
                                    issue.get("sequence_id").and_then(|s| s.as_u64()),
                                ) {
                                    seq_map.insert(title.to_string(), seq);
                                }
                            }
                        }
                    }
                }
                Ok(r) => warn!(
                    "trial bootstrap apply {} → {}: {}",
                    url,
                    r.status(),
                    r.text().await.unwrap_or_default()
                ),
                Err(e) => warn!("trial bootstrap apply {url}: {e}"),
            }
        }
        Ok(seq_map)
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
