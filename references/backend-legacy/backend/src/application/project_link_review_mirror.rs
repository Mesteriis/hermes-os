use sqlx::{Postgres, Transaction};

use crate::domains::projects::link_reviews::models::{
    ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::workflows::review_mirror::ReviewMirrorError;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn ensure_project_link_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
    graph_node_id: Option<&str>,
) -> Result<(), ReviewMirrorError> {
    crate::workflows::review_mirror::project_link::ensure_project_link_candidate_review_item(
        pool,
        project_id,
        target_kind,
        target_id,
        title,
        summary,
        confidence,
        observation_id,
        graph_node_id,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn sync_project_link_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    review_state: ProjectLinkReviewState,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    crate::workflows::review_mirror::project_link::sync_project_link_review_state_in_transaction(
        transaction,
        project_id,
        target_kind,
        target_id,
        review_state,
        title,
        summary,
        confidence,
        observation_id,
    )
    .await
}
