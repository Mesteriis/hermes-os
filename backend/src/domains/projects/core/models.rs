use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::domains::projects::link_reviews::ProjectLinkReviewState;

use super::errors::ProjectStoreError;
use super::validation::validate_non_empty;

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

    pub(super) fn validate(&self) -> Result<ValidatedProject, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", &self.project_id)?;
        let name = validate_non_empty("name", &self.name)?;
        let kind = validate_non_empty("kind", &self.kind)?;
        let status = validate_non_empty("status", &self.status)?;
        let description = validate_non_empty("description", &self.description)?;
        let owner_display_name =
            validate_non_empty("owner_display_name", &self.owner_display_name)?;
        if !(0..=100).contains(&self.progress_percent) {
            return Err(ProjectStoreError::InvalidProgress(self.progress_percent));
        }

        let mut seen = BTreeSet::new();
        let mut keywords = Vec::new();
        for keyword in &self.keywords {
            let keyword = validate_non_empty("keyword", keyword)?;
            if seen.insert(keyword.to_ascii_lowercase()) {
                keywords.push(keyword);
            }
        }
        if keywords.is_empty() {
            return Err(ProjectStoreError::NoKeywords);
        }

        Ok(ValidatedProject {
            project_id,
            name,
            kind,
            status,
            description,
            owner_display_name,
            progress_percent: self.progress_percent,
            start_date: self.start_date,
            target_date: self.target_date,
            keywords,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ValidatedProject {
    pub(super) project_id: String,
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) status: String,
    pub(super) description: String,
    pub(super) owner_display_name: String,
    pub(super) progress_percent: i32,
    pub(super) start_date: Option<NaiveDate>,
    pub(super) target_date: Option<NaiveDate>,
    pub(super) keywords: Vec<String>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectProjectionSource {
    pub project: Project,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMatchedMessage {
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
pub(crate) struct ProjectMatchedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub observation_id: String,
    pub source_fingerprint: String,
    pub imported_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}
