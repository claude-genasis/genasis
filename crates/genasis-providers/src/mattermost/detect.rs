//! Detect which Mattermost flavor a given URL exposes.

use reqwest::Client;

use genasis_core::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFlavor {
    Upstream,
    AgentAware,
}

pub async fn detect(base_url: &str) -> Result<DetectedFlavor> {
    let url = format!("{}/api/v4/system/ping", base_url.trim_end_matches('/'));
    let resp = Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Provider(format!("mm detect: {e}")))?;
    if resp
        .headers()
        .get("x-genasis-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(DetectedFlavor::AgentAware);
    }
    Ok(DetectedFlavor::Upstream)
}
