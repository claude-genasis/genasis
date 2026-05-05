//! GitHub Releases registry — fetch agent catalog tarballs.
//!
//! Release asset naming: `agents-v{version}.tar.gz`
//! Tag naming: `agents-v{version}`

use genasis_core::error::{Error, Result};

/// Check the latest agents catalog version available at the registry.
///
/// Queries the GitHub API for releases matching `agents-v*` tags and
/// returns the highest semver version string (without the `v` prefix).
pub fn check_latest(registry_url: &str) -> Result<String> {
    // Parse owner/repo from registry URL.
    // Expected format: "https://github.com/{owner}/{repo}/releases"
    let api_url = registry_to_api_url(registry_url)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("genasis-cli")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| Error::Config(format!("HTTP client build error: {e}")))?;

    let resp = client
        .get(&format!("{api_url}?per_page=20"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| Error::Provider(format!("registry check failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(Error::Provider(format!(
            "registry responded with status {}",
            resp.status()
        )));
    }

    let releases: Vec<serde_json::Value> = resp
        .json()
        .map_err(|e| Error::Provider(format!("registry JSON parse: {e}")))?;

    // Find the latest agents-v* tag.
    let mut latest: Option<String> = None;
    for release in &releases {
        if let Some(tag) = release.get("tag_name").and_then(|t| t.as_str()) {
            if let Some(ver) = tag.strip_prefix("agents-v") {
                if latest.as_ref().map_or(true, |l| ver > l.as_str()) {
                    latest = Some(ver.to_string());
                }
            }
        }
    }

    latest.ok_or_else(|| Error::Provider("no agents-v* releases found at registry".into()))
}

/// Download the agents catalog tarball for a specific version.
///
/// Returns the raw bytes of `agents-v{version}.tar.gz`.
pub fn fetch_tarball(registry_url: &str, version: &str) -> Result<Vec<u8>> {
    let api_url = registry_to_api_url(registry_url)?;
    let tag = format!("agents-v{version}");
    let asset_name = format!("agents-v{version}.tar.gz");

    let client = reqwest::blocking::Client::builder()
        .user_agent("genasis-cli")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| Error::Config(format!("HTTP client build error: {e}")))?;

    // Get the release by tag to find asset download URL.
    let release_url = format!(
        "{}/tags/{tag}",
        api_url.trim_end_matches("/releases").to_string() + "/releases"
    );
    let resp = client
        .get(&release_url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| Error::Provider(format!("fetch release info failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(Error::Provider(format!(
            "release {} not found (status {})",
            tag,
            resp.status()
        )));
    }

    let release: serde_json::Value = resp
        .json()
        .map_err(|e| Error::Provider(format!("release JSON parse: {e}")))?;

    // Find the asset URL.
    let assets = release
        .get("assets")
        .and_then(|a| a.as_array())
        .ok_or_else(|| Error::Provider("release has no assets array".into()))?;

    let asset_url = assets
        .iter()
        .find_map(|a| {
            let name = a.get("name")?.as_str()?;
            if name == asset_name {
                a.get("browser_download_url")?
                    .as_str()
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| Error::Provider(format!("asset {asset_name} not found in release {tag}")))?;

    // Download the tarball.
    let resp = client
        .get(&asset_url)
        .send()
        .map_err(|e| Error::Provider(format!("tarball download failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(Error::Provider(format!(
            "tarball download returned status {}",
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .map_err(|e| Error::Provider(format!("tarball read error: {e}")))?;

    Ok(bytes.to_vec())
}

/// Convert a registry URL like `https://github.com/owner/repo/releases`
/// to the GitHub API endpoint `https://api.github.com/repos/owner/repo/releases`.
fn registry_to_api_url(registry_url: &str) -> Result<String> {
    // Handle both "https://github.com/owner/repo/releases" and
    // "https://github.com/owner/repo" formats.
    let trimmed = registry_url
        .trim_end_matches('/')
        .trim_end_matches("/releases");
    let path = trimmed.strip_prefix("https://github.com/").ok_or_else(|| {
        Error::Config(format!(
            "registry URL must start with https://github.com/: {registry_url}"
        ))
    })?;
    Ok(format!("https://api.github.com/repos/{path}/releases"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_url_conversion() {
        let url = "https://github.com/claude-genasis/genasis/releases";
        let api = registry_to_api_url(url).unwrap();
        assert_eq!(
            api,
            "https://api.github.com/repos/claude-genasis/genasis/releases"
        );
    }

    #[test]
    fn registry_url_without_releases_suffix() {
        let url = "https://github.com/claude-genasis/genasis";
        let api = registry_to_api_url(url).unwrap();
        assert_eq!(
            api,
            "https://api.github.com/repos/claude-genasis/genasis/releases"
        );
    }
}
