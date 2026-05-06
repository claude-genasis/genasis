//! Thin entry point for the init/attach wizard TUI.
//! Delegates to `genasis_tui::wizard::app::run()`.

use std::path::PathBuf;

use anyhow::Result;
use genasis_tui::wizard::state::WizardMode;

pub async fn run_init_tui(project_root: PathBuf, non_interactive: bool) -> Result<()> {
    genasis_tui::wizard::app::run(WizardMode::Init, project_root, non_interactive)
        .await
        .map_err(Into::into)
}

pub async fn run_attach_tui(project_root: PathBuf, non_interactive: bool) -> Result<()> {
    genasis_tui::wizard::app::run(WizardMode::Attach, project_root, non_interactive)
        .await
        .map_err(Into::into)
}
