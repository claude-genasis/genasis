//! Detect which Plane flavor a given URL exposes.
//!
//! Strategy:
//! 1. `GET /api/v1/health/`.
//! 2. If response headers include `x-genasis-agent: true` → `agent-aware`.
//! 3. Else if response body JSON contains `"flavor": "agent-aware"` → same.
//! 4. Otherwise → `upstream`.

use reqwest::Client;

use genasis_core::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFlavor {
    Upstream,
    AgentAware,
}

pub async fn detect(base_url: &str) -> Result<DetectedFlavor> {
    let url = format!("{}/api/v1/health/", base_url.trim_end_matches('/'));
    let resp = Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Provider(format!("plane detect: {e}")))?;
    if resp
        .headers()
        .get("x-genasis-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(DetectedFlavor::AgentAware);
    }
    let text = resp.text().await.unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
        if v.get("flavor").and_then(|f| f.as_str()) == Some("agent-aware") {
            return Ok(DetectedFlavor::AgentAware);
        }
    }
    Ok(DetectedFlavor::Upstream)
}
