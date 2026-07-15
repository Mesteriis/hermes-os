use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use std::{future::Future, pin::Pin};

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectLinkCandidate {
    pub target_kind: String,
    pub target_id: String,
    pub observation_id: String,
    pub account_id: Option<String>,
    pub source_fingerprint: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub occurred_at: DateTime<Utc>,
}

pub type ProjectCandidatesFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<ProjectLinkCandidate>, ProjectQueryError>> + Send + 'a>>;
pub trait ProjectCandidateReadPort: Send + Sync {
    fn candidates<'a>(&'a self, project_id: &'a str) -> ProjectCandidatesFuture<'a>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectUpsert {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectProjectionSource {
    pub project: Project,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectMatchedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectMatchedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub observation_id: String,
    pub source_fingerprint: String,
    pub imported_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}

pub type ProjectProjectionFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Vec<ProjectProjectionSource>, ProjectQueryError>> + Send + 'a>,
>;
pub type ProjectMessagesFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Vec<ProjectMatchedMessage>, ProjectQueryError>> + Send + 'a>,
>;
pub type ProjectDocumentsFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Vec<ProjectMatchedDocument>, ProjectQueryError>> + Send + 'a>,
>;

pub trait ProjectGraphReadPort: Send + Sync {
    fn projection_projects<'a>(&'a self) -> ProjectProjectionFuture<'a>;
    fn matching_messages<'a>(&'a self, project_id: &'a str) -> ProjectMessagesFuture<'a>;
    fn matching_documents<'a>(&'a self, project_id: &'a str) -> ProjectDocumentsFuture<'a>;
}

pub type ProjectWriteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Project, ProjectQueryError>> + Send + 'a>>;
pub trait ProjectWritePort: Send + Sync {
    fn upsert<'a>(&'a self, project: &'a ProjectUpsert) -> ProjectWriteFuture<'a>;
}

pub type ProjectListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ProjectListResponse, ProjectQueryError>> + Send + 'a>>;
pub type ProjectDetailFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<ProjectDetail>, ProjectQueryError>> + Send + 'a>>;
pub trait ProjectReadPort: Send + Sync {
    fn list<'a>(&'a self, limit: Option<i64>) -> ProjectListFuture<'a>;
    fn detail<'a>(&'a self, project_id: &'a str) -> ProjectDetailFuture<'a>;
}
#[derive(Debug, thiserror::Error)]
#[error("project query failed: {0}")]
pub struct ProjectQueryError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_list_response_keeps_stable_json_shape() {
        let response = ProjectListResponse { items: vec![] };
        let value = serde_json::to_value(response).expect("projects response must serialize");
        assert_eq!(value, serde_json::json!({ "items": [] }));
    }
}
