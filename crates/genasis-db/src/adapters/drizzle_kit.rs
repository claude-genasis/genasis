//! Drizzle Kit — when the user already has `drizzle-orm` + `drizzle.config.ts`
//! we delegate `migrate` and `diff` to `npx drizzle-kit`.

use std::path::Path;

use genasis_core::error::{Error, Result};
use tokio::process::Command;

pub fn detected(project_root: &Path) -> bool {
    project_root.join("drizzle.config.ts").is_file()
        || project_root.join("drizzle.config.js").is_file()
}

pub async fn apply(project_root: &Path) -> Result<String> {
    let out = Command::new("npx")
        .current_dir(project_root)
        .args(["drizzle-kit", "push"])
        .output()
        .await
        .map_err(|e| Error::Db(format!("npx drizzle-kit push: {e}")))?;
    if !out.status.success() {
        return Err(Error::Db(format!(
            "drizzle-kit push failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub async fn diff(project_root: &Path) -> Result<String> {
    let out = Command::new("npx")
        .current_dir(project_root)
        .args(["drizzle-kit", "generate"])
        .output()
        .await
        .map_err(|e| Error::Db(format!("npx drizzle-kit generate: {e}")))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detected_true_when_ts_config_present() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("drizzle.config.ts"), "// stub").unwrap();
        assert!(detected(dir.path()));
    }

    #[test]
    fn detected_true_when_js_config_present() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("drizzle.config.js"), "// stub").unwrap();
        assert!(detected(dir.path()));
    }

    #[test]
    fn detected_false_when_no_config() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "# project").unwrap();
        assert!(!detected(dir.path()));
    }
}
