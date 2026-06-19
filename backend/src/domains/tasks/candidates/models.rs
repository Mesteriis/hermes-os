use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::constants::{TASK_CANDIDATE_KIND_OBLIGATION_TASK, TASK_CANDIDATE_KIND_TASK};
use super::errors::TaskCandidateError;
use super::ids::task_candidate_id_from_source;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateSourceKind {
    Observation,
}

impl TaskCandidateSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observation => "observation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateKind {
    Task,
    ObligationTask,
}

impl TaskCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => TASK_CANDIDATE_KIND_TASK,
            Self::ObligationTask => TASK_CANDIDATE_KIND_OBLIGATION_TASK,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl TaskCandidateReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, TaskCandidateError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(TaskCandidateError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommand {
    pub command_id: String,
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommandResult {
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TaskCandidate {
    pub task_candidate_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub project_id: Option<String>,
    pub title: String,
    pub due_text: Option<String>,
    pub assignee_label: Option<String>,
    pub confidence: f64,
    pub review_state: String,
    pub evidence_excerpt: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct CandidatePayload {
    pub(crate) source_kind: TaskCandidateSourceKind,
    pub(crate) source_id: String,
    pub(crate) observation_id: Option<String>,
    pub(crate) candidate_kind: TaskCandidateKind,
    pub(crate) candidate_metadata: Value,
    pub(crate) project_id: Option<String>,
    pub(crate) title: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
    pub(crate) confidence: f64,
    pub(crate) evidence_excerpt: String,
}

impl CandidatePayload {
    pub(crate) fn task_candidate_id(&self) -> String {
        let source_id = self
            .observation_id
            .as_deref()
            .unwrap_or(self.source_id.as_str());
        task_candidate_id_from_source(self.source_kind.as_str(), source_id, &self.title)
    }
}

#[derive(Debug)]
pub(crate) struct StoredCandidateRow {
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) observation_id: Option<String>,
    pub(crate) candidate_kind: String,
    pub(crate) candidate_metadata: Value,
    pub(crate) project_id: Option<String>,
    pub(crate) title: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
    pub(crate) confidence: f64,
    pub(crate) evidence_excerpt: String,
}
