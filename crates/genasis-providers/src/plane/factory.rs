//! Build a `PlaneProvider` from configured flavor + credentials.

use std::sync::Arc;

use genasis_core::config::TrialConfig;
use genasis_core::error::{Error, Result};

use super::agent_aware::AgentAwarePlane;
use super::detect::{detect, DetectedFlavor};
use super::trial::TrialPlane;
use super::upstream::UpstreamPlane;
use super::PlaneProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlavorChoice {
    Upstream,
    AgentAware,
    Auto,
    /// Forwards every call to a running trial-app instance over HTTP.
    /// The destination URL and shared secret are read from the
    /// `[trial]` config section, NOT from the per-provider `[plane]`
    /// fields, so a single source of truth governs trial routing.
    Trial,
}

impl FlavorChoice {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "upstream" => Ok(Self::Upstream),
            "agent-aware" | "agent_aware" => Ok(Self::AgentAware),
            "auto" => Ok(Self::Auto),
            "trial" => Ok(Self::Trial),
            other => Err(Error::Config(format!(
                "unknown plane flavor `{other}` (allowed: upstream, agent-aware, auto, trial)"
            ))),
        }
    }
}

/// Build a `PlaneProvider` for the requested flavor.
///
/// `base_url`, `workspace_slug`, and `api_key` are used for `Upstream` /
/// `AgentAware` / `Auto` flavors. For `Trial`, those fields are ignored
/// and the provider is constructed from `trial` (which is the `[trial]`
/// section in `genasis.toml`). Passing `Trial` without a populated
/// `trial` argument is a configuration error.
pub async fn build(
    flavor: FlavorChoice,
    base_url: &str,
    workspace_slug: &str,
    api_key: &str,
    trial: Option<&TrialConfig>,
) -> Result<Arc<dyn PlaneProvider>> {
    let resolved = match flavor {
        FlavorChoice::Auto => match detect(base_url).await? {
            DetectedFlavor::Upstream => FlavorChoice::Upstream,
            DetectedFlavor::AgentAware => FlavorChoice::AgentAware,
        },
        other => other,
    };
    Ok(match resolved {
        FlavorChoice::Upstream => Arc::new(UpstreamPlane::new(base_url, workspace_slug, api_key)),
        FlavorChoice::AgentAware => {
            Arc::new(AgentAwarePlane::new(base_url, workspace_slug, api_key))
        }
        FlavorChoice::Trial => {
            let t = trial.ok_or_else(|| {
                Error::Config(
                    "plane flavor=\"trial\" requires the [trial] section in genasis.toml".into(),
                )
            })?;
            if !t.enabled {
                return Err(Error::Config(
                    "plane flavor=\"trial\" but [trial] enabled = false; \
                     set enabled = true or change flavor"
                        .into(),
                ));
            }
            Arc::new(TrialPlane::new(&t.url, &t.shared_secret))
        }
        FlavorChoice::Auto => unreachable!("auto resolved above"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_trial() -> TrialConfig {
        TrialConfig {
            enabled: true,
            url: "http://localhost:3000".into(),
            shared_secret: "trialsecret".into(),
            team_token: None,
        }
    }

    #[test]
    fn flavor_parse_known_values() {
        assert_eq!(
            FlavorChoice::parse("upstream").unwrap(),
            FlavorChoice::Upstream
        );
        assert_eq!(
            FlavorChoice::parse("agent-aware").unwrap(),
            FlavorChoice::AgentAware
        );
        assert_eq!(FlavorChoice::parse("auto").unwrap(), FlavorChoice::Auto);
        assert_eq!(FlavorChoice::parse("trial").unwrap(), FlavorChoice::Trial);
    }

    #[test]
    fn flavor_parse_rejects_unknown() {
        assert!(FlavorChoice::parse("plane.so").is_err());
    }

    #[tokio::test]
    async fn build_upstream_ignores_trial() {
        let out = build(
            FlavorChoice::Upstream,
            "http://plane.example",
            "ws",
            "key",
            None,
        )
        .await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn build_agent_aware_ignores_trial() {
        let out = build(
            FlavorChoice::AgentAware,
            "http://plane.example",
            "ws",
            "key",
            None,
        )
        .await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn build_trial_requires_trial_section() {
        match build(FlavorChoice::Trial, "ignored", "ignored", "ignored", None).await {
            Err(Error::Config(_)) => {}
            Err(other) => panic!("expected Config error, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn build_trial_requires_enabled_true() {
        let mut t = enabled_trial();
        t.enabled = false;
        match build(
            FlavorChoice::Trial,
            "ignored",
            "ignored",
            "ignored",
            Some(&t),
        )
        .await
        {
            Err(Error::Config(msg)) => assert!(msg.contains("enabled"), "msg = {msg}"),
            Err(other) => panic!("expected Config error, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn build_trial_succeeds_when_enabled() {
        let t = enabled_trial();
        let out = build(
            FlavorChoice::Trial,
            "ignored",
            "ignored",
            "ignored",
            Some(&t),
        )
        .await;
        assert!(out.is_ok());
    }
}
