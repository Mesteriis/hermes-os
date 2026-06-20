use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionKind {
    Reply,
    CreateTask,
    CreateNote,
    CreateDocument,
    CreateEvent,
    LinkDocument,
    CreateContact,
    Archive,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionSource {
    pub(super) kind: String,
    pub(super) id: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct WorkflowActionInput {
    pub(super) title: Option<String>,
    pub(super) body: Option<String>,
    pub(super) email: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) starts_at: Option<DateTime<Utc>>,
    pub(super) ends_at: Option<DateTime<Utc>>,
    pub(super) due_at: Option<DateTime<Utc>>,
    pub(super) document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionRequest {
    pub(super) command_id: String,
    pub(super) action: WorkflowActionKind,
    pub(super) source: Option<WorkflowActionSource>,
    pub(super) input: Option<WorkflowActionInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionStatus {
    Created,
    Updated,
    Linked,
    Opened,
    Archived,
    Noop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionTargetKind {
    Compose,
    Message,
    Task,
    Document,
    CalendarEvent,
    Person,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionTarget {
    pub(super) kind: WorkflowActionTargetKind,
    pub(super) id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source_id: Option<String>,
    pub(super) confidence: Option<f64>,
    pub(super) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionResponse {
    pub(super) command_id: String,
    pub(super) event_id: String,
    pub(super) action: WorkflowActionKind,
    pub(super) status: WorkflowActionStatus,
    pub(super) target: WorkflowActionTarget,
    pub(super) provenance: WorkflowActionProvenance,
}
