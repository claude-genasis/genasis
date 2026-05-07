//! Build a `PlaneProvider` from configured flavor + credentials.

use std::sync::Arc;

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
    /// `base_url` becomes the trial-app URL; `api_key` becomes the
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
                "unknown plane flavor `{other}` (allowed: upstream, agent-aware, auto, trial)"
            ))),
        }
    }
}

pub async fn build(
    flavor: FlavorChoice,
    base_url: &str,
    workspace_slug: &str,
    api_key: &str,
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
        FlavorChoice::Trial => Arc::new(TrialPlane::new(base_url, api_key)),
        FlavorChoice::Auto => unreachable!("auto resolved above"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
