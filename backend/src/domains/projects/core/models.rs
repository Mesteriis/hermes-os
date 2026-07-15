use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProject {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub keywords: Vec<String>,
}

impl NewProject {
    pub fn active(
        project_id: impl Into<String>,
        name: impl Into<String>,
        kind: impl Into<String>,
        description: impl Into<String>,
        owner_display_name: impl Into<String>,
        keywords: Vec<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            name: name.into(),
            kind: kind.into(),
            status: "active".to_owned(),
            description: description.into(),
            owner_display_name: owner_display_name.into(),
            progress_percent: 0,
            start_date: None,
            target_date: None,
            keywords,
        }
    }

    pub fn progress(mut self, progress_percent: i32) -> Self {
        self.progress_percent = progress_percent;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectStats {
    pub message_count: i64,
    pub document_count: i64,
    pub persona_count: i64,
    #[deprecated(note = "Use persona_count; kept for compatibility with existing clients.")]
    pub people_count: i64,
    pub graph_connection_count: i64,
    pub latest_activity_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDetail {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
    pub timeline: Vec<ProjectTimelineItem>,
    pub key_personas: Vec<ProjectPersonaSummary>,
    #[deprecated(note = "Use key_personas; kept for compatibility with existing clients.")]
    pub key_people: Vec<ProjectPersonaSummary>,
    pub recent_messages: Vec<ProjectMessageSummary>,
    pub documents: Vec<ProjectDocumentSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectTimelineItem {
    pub item_kind: String,
    pub item_id: String,
    pub title: String,
    pub subtitle: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectPersonaSummary {
    pub display_name: String,
    pub email_address: String,
    pub interaction_count: i64,
    pub last_interaction_at: Option<DateTime<Utc>>,
}

#[deprecated(note = "Use ProjectPersonaSummary.")]
pub type ProjectPersonSummary = ProjectPersonaSummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectMessageSummary {
    pub message_id: String,
    pub subject: String,
    pub sender: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDocumentSummary {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub observation_id: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectListResponse {
    pub items: Vec<ProjectSummary>,
}
