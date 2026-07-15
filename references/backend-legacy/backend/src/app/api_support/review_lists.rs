use super::*;

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidate {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) graph_node_id: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) source_label: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) review_state: String,
    pub(crate) evidence_excerpt: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidateListResponse {
    pub(crate) items: Vec<ProjectLinkCandidate>,
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateListResponse {
    pub(crate) items: Vec<TaskCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct AiRunsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct AiRunListResponse {
    pub(crate) items: Vec<AiAgentRun>,
}
