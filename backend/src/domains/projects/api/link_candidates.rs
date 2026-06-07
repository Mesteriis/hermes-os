use super::dto::{LinkCandidate, LinkCandidateList, LinkCandidatesQuery};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::projects::core::ProjectStore;
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewState, ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use axum::Json;
use axum::extract::{Path, Query, State};

fn preview(s: &str, n: usize) -> String {
    let mut p: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        p.push_str("...");
    }
    p
}

pub async fn link_candidates(
    State(state): State<AppState>,
    Path(pid): Path<String>,
    Query(q): Query<LinkCandidatesQuery>,
) -> Result<Json<LinkCandidateList>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let ps = ProjectStore::new(pool.clone());
    let rs = ProjectLinkReviewStore::new(pool);
    let mut items = Vec::new();

    for m in ps.matching_project_messages(&pid).await? {
        let review = rs
            .explicit_review(&pid, ProjectLinkTargetKind::Message, &m.message_id)
            .await?
            .map(|r| r.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);
        let occurred = m.occurred_at.unwrap_or(m.projected_at);
        items.push(LinkCandidate {
            project_id: pid.clone(),
            target_kind: "message".into(),
            target_id: m.message_id.clone(),
            graph_node_id: node_id(GraphNodeKind::Message, &m.message_id),
            title: preview(&m.subject, 120),
            subtitle: m.sender.clone(),
            source_label: m.account_id.clone(),
            occurred_at: occurred,
            review_state: review.as_str().into(),
            evidence_excerpt: Some(preview(&m.sender, 140)),
        });
    }
    for d in ps.matching_project_documents(&pid).await? {
        let review = rs
            .explicit_review(&pid, ProjectLinkTargetKind::Document, &d.document_id)
            .await?
            .map(|r| r.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);
        let title = preview(&d.title, 140);
        items.push(LinkCandidate {
            project_id: pid.clone(),
            target_kind: "document".into(),
            target_id: d.document_id.clone(),
            graph_node_id: node_id(GraphNodeKind::Document, &d.document_id),
            title: title.clone(),
            subtitle: d.document_kind.clone(),
            source_label: d.source_fingerprint.clone(),
            occurred_at: d.imported_at,
            review_state: review.as_str().into(),
            evidence_excerpt: Some(title),
        });
    }
    items.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    items.truncate(q.limit.unwrap_or(25));
    Ok(Json(LinkCandidateList { items }))
}
