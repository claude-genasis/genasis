//! WizardStep enum and navigation helpers.

use super::state::WizardMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WizardStep {
    Env,
    Lang,
    Team,
    Connect,
    Overlay,
    Done,
}

impl WizardStep {
    pub const ALL: [WizardStep; 6] = [
        Self::Env,
        Self::Lang,
        Self::Team,
        Self::Connect,
        Self::Overlay,
        Self::Done,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Env => 0,
            Self::Lang => 1,
            Self::Team => 2,
            Self::Connect => 3,
            Self::Overlay => 4,
            Self::Done => 5,
        }
    }

    pub fn from_index(i: usize) -> Option<Self> {
        Self::ALL.get(i).copied()
    }

    pub fn next(self) -> Option<Self> {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(self) -> Option<Self> {
        if self.index() == 0 {
            None
        } else {
            Self::from_index(self.index() - 1)
        }
    }

    pub fn label(self, mode: WizardMode) -> &'static str {
        match (self, mode) {
            (Self::Env, _) => "Env",
            (Self::Lang, _) => "Lang",
            (Self::Team, WizardMode::Init) => "Team",
            (Self::Team, WizardMode::Attach) => "Detect",
            (Self::Connect, _) => "Connect",
            (Self::Overlay, _) => "Overlay",
            (Self::Done, _) => "Done",
        }
    }

    pub fn number(self) -> char {
        match self {
            Self::Env => '1',
            Self::Lang => '2',
            Self::Team => '3',
            Self::Connect => '4',
            Self::Overlay => '5',
            Self::Done => '6',
        }
    }
}
