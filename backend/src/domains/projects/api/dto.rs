use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ProjectListResponse {
    pub items: Vec<crate::domains::projects::core::ProjectSummary>,
}
#[derive(Deserialize)]
pub struct ProjectsQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct LinkCandidate {
    pub project_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub graph_node_id: String,
    pub title: String,
    pub subtitle: String,
    pub source_label: String,
    pub occurred_at: DateTime<Utc>,
    pub review_state: String,
    pub evidence_excerpt: Option<String>,
}
#[derive(Serialize)]
pub struct LinkCandidateList {
    pub items: Vec<LinkCandidate>,
}
#[derive(Deserialize)]
pub struct LinkCandidatesQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct LinkReviewRequest {
    pub target_kind: String,
    pub target_id: String,
    pub review_state: String,
}
#[derive(Serialize)]
pub struct LinkReviewResponse {
    pub project_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub review_state: String,
}
