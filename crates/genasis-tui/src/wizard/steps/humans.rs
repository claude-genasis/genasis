//! Step 5: Human roster CRUD. Loads `[[humans]]` from
//! `genasis.toml` on enter, lets the user add/edit/delete entries via
//! a simple form, persists changes back on advance, and triggers an
//! out-of-process `genasis humans sync` when `s` is pressed.
//!
//! See ADR-014.

use crossterm::event::KeyCode;

use genasis_core::config::{Config, HumanEntry, HumansLock, CONFIG_FILE_NAME};

use crate::wizard::state::{AsyncResult, HumansForm, HumansRow, WizardState};

/// Called when the Humans step becomes active.
pub fn on_enter(state: &mut WizardState) {
    if state.humans.loaded {
        return;
    }
    state.humans.loaded = true;
    let project_root = state.project_root.clone();
    let tx = state.async_tx.clone();
    tokio::spawn(async move {
        let rows = load_rows(&project_root);
        let _ = tx.send(AsyncResult::HumansLoaded(rows));
    });
}

pub fn handle_key(state: &mut WizardState, code: KeyCode) {
    // Lazy-load roster on first interaction with this step.
    if !state.humans.loaded {
        on_enter(state);
    }

    if let Some(form) = state.humans.form.take() {
        handle_form_key(state, form, code);
        return;
    }

    match code {
        KeyCode::Up => {
            if state.humans.selected > 0 {
                state.humans.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.humans.selected + 1 < state.humans.entries.len() {
                state.humans.selected += 1;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            state.humans.form = Some(HumansForm {
                role: "stakeholder".into(),
                ..Default::default()
            });
            state.humans.status_line = "add — Tab cycles fields, Enter saves, Esc cancels".into();
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if let Some(row) = state.humans.entries.get(state.humans.selected).cloned() {
                state.humans.form = Some(HumansForm {
                    editing_email: Some(row.email.clone()),
                    name: row.name,
                    email: row.email,
                    role: row.role,
                    mm_username: row.mm_username,
                    locale: row.locale,
                    focus: 0,
                    error: String::new(),
                });
                state.humans.status_line =
                    "edit — Tab cycles fields, Enter saves, Esc cancels".into();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(row) = state.humans.entries.get(state.humans.selected).cloned() {
                if let Err(e) = persist_remove(state, &row.email) {
                    state.humans.status_line = format!("delete failed: {e}");
                } else {
                    if state.humans.selected >= state.humans.entries.len()
                        && state.humans.selected > 0
                    {
                        state.humans.selected -= 1;
                    }
                    state.humans.status_line = format!("- removed {}", row.email);
                }
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if state.humans.syncing {
                return;
            }
            state.humans.syncing = true;
            state.humans.status_line = "syncing humans (Mattermost + Plane)…".into();
            let project_root = state.project_root.clone();
            let tx = state.async_tx.clone();
            tokio::spawn(async move {
                let (ok, msg) = run_sync(&project_root).await;
                let _ = tx.send(AsyncResult::HumansSyncDone(ok, msg));
            });
        }
        KeyCode::Enter => {
            // Advance — but only if at least one human exists OR the user
            // explicitly skipped (we treat empty as a valid state for
            // small projects).
            let n = state.humans.entries.len();
            let summary = if n == 0 {
                "0 humans (skipped)".into()
            } else {
                format!("{n} humans")
            };
            state.advance(summary);
        }
        _ => {}
    }
}

fn handle_form_key(state: &mut WizardState, mut form: HumansForm, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            state.humans.form = None;
            state.humans.status_line = "(cancelled)".into();
            return;
        }
        KeyCode::Tab => {
            form.focus = (form.focus + 1) % 5;
        }
        KeyCode::BackTab => {
            form.focus = (form.focus + 4) % 5;
        }
        KeyCode::Enter => {
            // Validate.
            if form.name.trim().is_empty() {
                form.error = "name is required".into();
                state.humans.form = Some(form);
                return;
            }
            if !form.email.contains('@') || form.email.trim().len() < 3 {
                form.error = "email must contain @".into();
                state.humans.form = Some(form);
                return;
            }
            let entry = HumanEntry {
                name: form.name.trim().into(),
                email: form.email.trim().to_ascii_lowercase(),
                role: if form.role.trim().is_empty() {
                    "stakeholder".into()
                } else {
                    form.role.trim().into()
                },
                mm_username: form.mm_username.trim().into(),
                locale: form.locale.trim().into(),
            };
            match persist_upsert(state, entry.clone(), form.editing_email.as_deref()) {
                Ok(was_new) => {
                    state.humans.form = None;
                    state.humans.status_line = format!(
                        "{} {} ({})",
                        if was_new { "+ added" } else { "~ saved" },
                        entry.email,
                        entry.name
                    );
                }
                Err(e) => {
                    form.error = format!("save failed: {e}");
                    state.humans.form = Some(form);
                }
            }
            return;
        }
        KeyCode::Backspace => {
            field_pop(&mut form);
        }
        KeyCode::Char(c) => {
            field_push(&mut form, c);
        }
        _ => {}
    }
    state.humans.form = Some(form);
}

fn field_push(form: &mut HumansForm, c: char) {
    let target = match form.focus {
        0 => &mut form.name,
        1 => &mut form.email,
        2 => &mut form.role,
        3 => &mut form.mm_username,
        _ => &mut form.locale,
    };
    target.push(c);
}

fn field_pop(form: &mut HumansForm) {
    let target = match form.focus {
        0 => &mut form.name,
        1 => &mut form.email,
        2 => &mut form.role,
        3 => &mut form.mm_username,
        _ => &mut form.locale,
    };
    target.pop();
}

pub fn handle_async(state: &mut WizardState, result: AsyncResult) {
    match result {
        AsyncResult::HumansLoaded(rows) => {
            state.humans.entries = rows;
            if state.humans.selected >= state.humans.entries.len()
                && !state.humans.entries.is_empty()
            {
                state.humans.selected = state.humans.entries.len() - 1;
            }
        }
        AsyncResult::HumansSyncDone(ok, msg) => {
            state.humans.syncing = false;
            state.humans.status_line = msg;
            if ok {
                // Reload rows so the "provisioned" column is refreshed.
                state.humans.entries = load_rows(&state.project_root);
            }
        }
        _ => {}
    }
}

fn load_rows(project_root: &std::path::Path) -> Vec<HumansRow> {
    let cfg_path = project_root.join(CONFIG_FILE_NAME);
    let cfg = match Config::load(&cfg_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lock = HumansLock::load(project_root).unwrap_or_default();
    cfg.humans
        .into_iter()
        .map(|h| {
            let provisioned = lock
                .find(&h.email)
                .map(|e| {
                    if !e.mm_user_id.is_empty() && !e.plane_user_id.is_empty() {
                        "mm+plane"
                    } else if !e.mm_user_id.is_empty() {
                        "mm"
                    } else if !e.plane_user_id.is_empty() {
                        "plane"
                    } else {
                        "no"
                    }
                })
                .unwrap_or("no");
            HumansRow {
                name: h.name,
                email: h.email,
                role: h.role,
                mm_username: h.mm_username,
                locale: h.locale,
                provisioned: provisioned.into(),
            }
        })
        .collect()
}

fn persist_upsert(
    state: &mut WizardState,
    entry: HumanEntry,
    editing_email: Option<&str>,
) -> Result<bool, String> {
    let cfg_path = state.project_root.join(CONFIG_FILE_NAME);
    let mut cfg = Config::load(&cfg_path).map_err(|e| e.to_string())?;
    if let Some(old_email) = editing_email {
        if !old_email.eq_ignore_ascii_case(&entry.email) {
            cfg.remove_human(old_email);
        }
    }
    let was_new = cfg.upsert_human(entry);
    cfg.save(&cfg_path).map_err(|e| e.to_string())?;
    state.humans.entries = load_rows(&state.project_root);
    Ok(was_new)
}

fn persist_remove(state: &mut WizardState, email: &str) -> Result<(), String> {
    let cfg_path = state.project_root.join(CONFIG_FILE_NAME);
    let mut cfg = Config::load(&cfg_path).map_err(|e| e.to_string())?;
    cfg.remove_human(email);
    cfg.save(&cfg_path).map_err(|e| e.to_string())?;
    let mut lock = HumansLock::load(&state.project_root).map_err(|e| e.to_string())?;
    if lock.remove(email) {
        lock.save(&state.project_root).map_err(|e| e.to_string())?;
    }
    state.humans.entries = load_rows(&state.project_root);
    Ok(())
}

/// Out-of-process call into `genasis humans sync`. We deliberately
/// shell out so the TUI does not have to re-implement provisioning
/// logic that already lives in the CLI crate (which depends on this
/// crate, so we can't import it).
async fn run_sync(project_root: &std::path::Path) -> (bool, String) {
    use tokio::process::Command;
    let bin = std::env::var("GENASIS_BIN").unwrap_or_else(|_| "genasis".into());
    let out = Command::new(&bin)
        .arg("humans")
        .arg("sync")
        .arg("--project")
        .arg(project_root)
        .output()
        .await;
    match out {
        Ok(o) if o.status.success() => (true, "sync ok — see `.genasis/humans.lock.toml`".into()),
        Ok(o) => (
            false,
            format!(
                "sync exit {}: {}",
                o.status,
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .last()
                    .unwrap_or("")
                    .to_string()
            ),
        ),
        Err(e) => (false, format!("sync spawn failed: {e}")),
    }
}
