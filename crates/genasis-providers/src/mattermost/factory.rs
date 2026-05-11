//! Build a `MattermostProvider` from configured flavor + credentials.

use std::sync::Arc;

use genasis_core::config::TrialConfig;
use genasis_core::error::{Error, Result};

use super::agent_aware::AgentAwareMattermost;
use super::detect::{detect, DetectedFlavor};
use super::trial::TrialMattermost;
use super::upstream::UpstreamMattermost;
use super::MattermostProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlavorChoice {
    Upstream,
    AgentAware,
    Auto,
    /// Forwards every call to a running trial-app instance over HTTP.
    /// The destination URL and shared secret are read from the
    /// `[trial]` config section, NOT from the per-provider `[mattermost]`
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
                "unknown mattermost flavor `{other}` (allowed: upstream, agent-aware, auto, trial)"
            ))),
        }
    }
}

/// Build a `MattermostProvider` for the requested flavor.
///
/// `base_url` and `admin_token` are used for `Upstream` / `AgentAware` /
/// `Auto` flavors. For `Trial`, both are ignored and the provider is
/// constructed from `trial` (which is the `[trial]` section in
/// `genasis.toml`). Passing `Trial` without a populated `trial` argument
/// is a configuration error.
pub async fn build(
    flavor: FlavorChoice,
    base_url: &str,
    admin_token: &str,
    trial: Option<&TrialConfig>,
) -> Result<Arc<dyn MattermostProvider>> {
    let resolved = match flavor {
        FlavorChoice::Auto => match detect(base_url).await? {
            DetectedFlavor::Upstream => FlavorChoice::Upstream,
            DetectedFlavor::AgentAware => FlavorChoice::AgentAware,
        },
        other => other,
    };
    Ok(match resolved {
        FlavorChoice::Upstream => Arc::new(UpstreamMattermost::new(base_url, admin_token)),
        FlavorChoice::AgentAware => Arc::new(AgentAwareMattermost::new(base_url, admin_token)),
        FlavorChoice::Trial => {
            let t = trial.ok_or_else(|| {
                Error::Config(
                    "mattermost flavor=\"trial\" requires the [trial] section in genasis.toml"
                        .into(),
                )
            })?;
            if !t.enabled {
                return Err(Error::Config(
                    "mattermost flavor=\"trial\" but [trial] enabled = false; \
                     set enabled = true or change flavor"
                        .into(),
                ));
            }
            // ADR-016 §3: scope every call into the tenant's sim
            // namespace via X-Genasis-Team-Token. Empty token =
            // trial-app falls through to DEFAULT_TEAM_TOKEN.
            let team_token = t.team_token.clone().unwrap_or_default();
            Arc::new(TrialMattermost::new(&t.url, &t.shared_secret, team_token))
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
            team_token: Some("test-token-abc".into()),
        }
    }

    #[test]
    fn flavor_parse() {
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
        assert!(FlavorChoice::parse("xxx").is_err());
    }

    #[tokio::test]
    async fn build_upstream_ignores_trial() {
        let out = build(FlavorChoice::Upstream, "http://mm.example", "tok", None).await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn build_agent_aware_ignores_trial() {
        let out = build(FlavorChoice::AgentAware, "http://mm.example", "tok", None).await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn build_trial_requires_trial_section() {
        match build(FlavorChoice::Trial, "ignored", "ignored", None).await {
            Err(Error::Config(_)) => {}
            Err(other) => panic!("expected Config error, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn build_trial_requires_enabled_true() {
        let mut t = enabled_trial();
        t.enabled = false;
        match build(FlavorChoice::Trial, "ignored", "ignored", Some(&t)).await {
            Err(Error::Config(msg)) => assert!(msg.contains("enabled"), "msg = {msg}"),
            Err(other) => panic!("expected Config error, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn build_trial_succeeds_when_enabled() {
        let t = enabled_trial();
        let out = build(FlavorChoice::Trial, "ignored", "ignored", Some(&t)).await;
        assert!(out.is_ok());
    }
}
