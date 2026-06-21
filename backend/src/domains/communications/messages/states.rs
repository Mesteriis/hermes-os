use serde::{Deserialize, Serialize};

use super::errors::MessageProjectionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    New,
    Reviewed,
    NeedsAction,
    Waiting,
    Done,
    Archived,
    Muted,
    Spam,
}

impl WorkflowState {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowState::New => "new",
            WorkflowState::Reviewed => "reviewed",
            WorkflowState::NeedsAction => "needs_action",
            WorkflowState::Waiting => "waiting",
            WorkflowState::Done => "done",
            WorkflowState::Archived => "archived",
            WorkflowState::Muted => "muted",
            WorkflowState::Spam => "spam",
        }
    }

    pub fn valid_transitions(&self) -> &[WorkflowState] {
        match self {
            WorkflowState::New => &[
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
                WorkflowState::Archived,
                WorkflowState::Muted,
                WorkflowState::Spam,
            ],
            WorkflowState::Reviewed => &[
                WorkflowState::New,
                WorkflowState::NeedsAction,
                WorkflowState::Waiting,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Muted,
                WorkflowState::Spam,
            ],
            WorkflowState::NeedsAction => &[
                WorkflowState::Waiting,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Reviewed,
            ],
            WorkflowState::Waiting => &[
                WorkflowState::NeedsAction,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Reviewed,
            ],
            WorkflowState::Done => &[
                WorkflowState::Archived,
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
            ],
            WorkflowState::Archived => &[
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
                WorkflowState::Done,
            ],
            WorkflowState::Muted => &[WorkflowState::Reviewed, WorkflowState::Archived],
            WorkflowState::Spam => &[
                WorkflowState::Reviewed,
                WorkflowState::Archived,
                WorkflowState::New,
            ],
        }
    }

    pub fn is_valid_transition(from: &Self, to: &Self) -> bool {
        from.valid_transitions().contains(to)
    }
}

impl std::str::FromStr for WorkflowState {
    type Err = MessageProjectionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "new" => Ok(WorkflowState::New),
            "reviewed" => Ok(WorkflowState::Reviewed),
            "needs_action" => Ok(WorkflowState::NeedsAction),
            "waiting" => Ok(WorkflowState::Waiting),
            "done" => Ok(WorkflowState::Done),
            "archived" => Ok(WorkflowState::Archived),
            "muted" => Ok(WorkflowState::Muted),
            "spam" => Ok(WorkflowState::Spam),
            _ => Err(MessageProjectionError::InvalidWorkflowState(
                value.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalMessageState {
    Active,
    Trash,
    All,
}

impl LocalMessageState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LocalMessageState::Active => "active",
            LocalMessageState::Trash => "trash",
            LocalMessageState::All => "all",
        }
    }

    pub(crate) fn persisted_filter(&self) -> Option<&'static str> {
        match self {
            LocalMessageState::Active => Some("active"),
            LocalMessageState::Trash => Some("trash"),
            LocalMessageState::All => None,
        }
    }
}

impl std::str::FromStr for LocalMessageState {
    type Err = MessageProjectionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "active" => Ok(LocalMessageState::Active),
            "trash" => Ok(LocalMessageState::Trash),
            "all" => Ok(LocalMessageState::All),
            _ => Err(MessageProjectionError::InvalidLocalState(value.to_owned())),
        }
    }
}
