//! Build a `MattermostProvider` from configured flavor + credentials.

use std::sync::Arc;

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
    /// `base_url` becomes the trial-app URL; `admin_token` becomes the
    /// shared secret sent in `X-Genasis-Trial-Secret`.
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

pub async fn build(
    flavor: FlavorChoice,
    base_url: &str,
    admin_token: &str,
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
        FlavorChoice::Trial => Arc::new(TrialMattermost::new(base_url, admin_token)),
        FlavorChoice::Auto => unreachable!("auto resolved above"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
