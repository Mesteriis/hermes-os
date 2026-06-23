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
    pub(crate) kind: String,
    pub(crate) id: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct WorkflowActionInput {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) starts_at: Option<DateTime<Utc>>,
    pub(crate) ends_at: Option<DateTime<Utc>>,
    pub(crate) due_at: Option<DateTime<Utc>>,
    pub(crate) document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionRequest {
    pub(crate) command_id: String,
    pub(crate) action: WorkflowActionKind,
    pub(crate) source: Option<WorkflowActionSource>,
    pub(crate) input: Option<WorkflowActionInput>,
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
    pub(crate) kind: WorkflowActionTargetKind,
    pub(crate) id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_id: Option<String>,
    pub(crate) confidence: Option<f64>,
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionResponse {
    pub(crate) command_id: String,
    pub(crate) event_id: String,
    pub(crate) action: WorkflowActionKind,
    pub(crate) status: WorkflowActionStatus,
    pub(crate) target: WorkflowActionTarget,
    pub(crate) provenance: WorkflowActionProvenance,
}
