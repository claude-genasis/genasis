//! Central wizard state. Owned by the event loop; widgets receive `&WizardState`.

use std::path::PathBuf;

use tokio::sync::mpsc;

use super::step::WizardStep;

/// Which mode launched the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardMode {
    Init,
    Attach,
}

/// Completion status of a single step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Active,
    Done,
    DoneWithWarnings,
    Failed,
}

impl StepStatus {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Done => "✅",
            Self::DoneWithWarnings => "⚠",
            Self::Active => "●",
            Self::Failed => "✗",
            Self::Pending => "○",
        }
    }

    pub fn is_complete(self) -> bool {
        matches!(self, Self::Done | Self::DoneWithWarnings)
    }
}

/// Per-step metadata shown in the tab bar.
#[derive(Debug, Clone)]
pub struct StepMeta {
    pub status: StepStatus,
    pub summary: String,
}

impl Default for StepMeta {
    fn default() -> Self {
        Self {
            status: StepStatus::Pending,
            summary: String::new(),
        }
    }
}

// ── Per-step sub-states ─────────────────────────────────────────────

/// Step 1: Environment prerequisite checks.
#[derive(Debug, Default)]
pub struct EnvStepState {
    pub checks: Vec<PrereqCheck>,
    pub selected: usize,
    pub scanning: bool,
}

#[derive(Debug, Clone)]
pub struct PrereqCheck {
    pub tool: String,
    pub required: bool,
    pub found: bool,
    pub version: String,
}

/// Step 2: Language selection.
#[derive(Debug)]
pub struct LangStepState {
    pub cursor: usize, // 0=en, 1=ko
    pub confirmed: bool,
}

impl Default for LangStepState {
    fn default() -> Self {
        Self {
            cursor: 0,
            confirmed: false,
        }
    }
}

/// Step 3: Team bootstrap (init) or agent detection (attach).
#[derive(Debug, Default)]
pub struct TeamStepState {
    pub agents_found: Vec<AgentEntry>,
    pub agents_created: usize,
    pub agents_total: usize,
    pub selected: usize,
    pub scanning: bool,
    pub applying: bool,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub role: String,
    pub status: AgentEntryStatus,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentEntryStatus {
    Pending,
    Creating,
    Created,
    Detected,
    Skipped,
}

/// Step 4: Plane + Mattermost connection.
#[derive(Debug, Default)]
pub struct ConnectStepState {
    pub plane_url: String,
    pub plane_workspace: String,
    pub plane_status: ConnStatus,
    pub mm_url: String,
    pub mm_status: ConnStatus,
    pub bots_provisioned: usize,
    pub bots_total: usize,
    pub focus: usize, // 0=plane_url, 1=plane_ws, 2=mm_url, 3=probe_btn
    pub probing: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConnStatus {
    #[default]
    Untested,
    Testing,
    Ok,
    Failed,
    Skipped,
}

/// Step 5: Overlay injection plan + apply.
#[derive(Debug, Default)]
pub struct OverlayStepState {
    pub files_total: usize,
    pub files_injected: usize,
    pub conflicts: usize,
    pub diff_text: String,
    pub show_diff: bool,
    pub planning: bool,
    pub applying: bool,
    pub applied: bool,
    pub scroll: u16,
}

/// Step 6: Done / summary / smoke test.
#[derive(Debug, Default)]
pub struct DoneStepState {
    pub summary_lines: Vec<(String, String)>, // (label, value)
    pub smoke_status: SmokeTestStatus,
    pub smoke_output: Vec<String>,
    pub rollback_available: bool,
    pub button_focus: usize, // 0=run, 1=skip, 2=rollback, 3=finish
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SmokeTestStatus {
    #[default]
    NotRun,
    Running,
    Passed,
    Failed,
}

// ── Async messages ──────────────────────────────────────────────────

/// Messages sent from background tasks to the event loop.
#[derive(Debug)]
pub enum AsyncResult {
    EnvScanComplete(Vec<PrereqCheck>),
    TeamScanComplete(Vec<AgentEntry>),
    TeamBootstrapProgress(usize, usize),
    TeamBootstrapDone(Vec<AgentEntry>),
    PlaneProbeResult(bool, String),
    MmProbeResult(bool, String),
    OverlayPlanReady(usize, usize, String), // total, conflicts, diff_text
    OverlayApplied(usize),
    SmokeTestProgress(String),
    SmokeTestDone(bool),
    RollbackDone(bool),
}

// ── Top-level wizard state ──────────────────────────────────────────

/// The central state struct for the init/attach wizard.
pub struct WizardState {
    pub mode: WizardMode,
    pub current_step: WizardStep,
    pub steps: [StepMeta; 6],
    pub should_quit: bool,
    pub project_root: PathBuf,

    // Per-step state
    pub env: EnvStepState,
    pub lang: LangStepState,
    pub team: TeamStepState,
    pub connect: ConnectStepState,
    pub overlay: OverlayStepState,
    pub done: DoneStepState,

    // Async channel
    pub async_tx: mpsc::UnboundedSender<AsyncResult>,
    pub async_rx: mpsc::UnboundedReceiver<AsyncResult>,
}

impl WizardState {
    pub fn new(mode: WizardMode, project_root: PathBuf) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut steps: [StepMeta; 6] = Default::default();
        steps[0].status = StepStatus::Active;
        Self {
            mode,
            current_step: WizardStep::Env,
            steps,
            should_quit: false,
            project_root,
            env: EnvStepState::default(),
            lang: LangStepState::default(),
            team: TeamStepState::default(),
            connect: ConnectStepState::default(),
            overlay: OverlayStepState::default(),
            done: DoneStepState::default(),
            async_tx: tx,
            async_rx: rx,
        }
    }

    /// Navigate to a step (only if reachable).
    pub fn go_to(&mut self, target: WizardStep) {
        let idx = target.index();
        // Can only go to completed steps or the next pending one.
        if idx <= self.current_step.index() || self.steps[idx - 1].status.is_complete() {
            self.current_step = target;
        }
    }

    /// Advance to the next step, marking current as done.
    pub fn advance(&mut self, summary: String) {
        let idx = self.current_step.index();
        self.steps[idx].status = StepStatus::Done;
        self.steps[idx].summary = summary;
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            self.steps[next.index()].status = StepStatus::Active;
        }
    }

    /// Mark current step as failed.
    pub fn fail_current(&mut self, summary: String) {
        let idx = self.current_step.index();
        self.steps[idx].status = StepStatus::Failed;
        self.steps[idx].summary = summary;
    }

    /// Selected language code.
    pub fn lang_code(&self) -> &str {
        if self.lang.cursor == 0 {
            "en"
        } else {
            "ko"
        }
    }
}
