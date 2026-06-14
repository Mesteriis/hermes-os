use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ProjectLinkReviewError;
use super::models::{
    ProjectLinkReview, ProjectLinkReviewState, ProjectLinkTargetKind, ProjectReviewedTarget,
};

pub(crate) fn row_to_project_link_review(
    row: PgRow,
) -> Result<ProjectLinkReview, ProjectLinkReviewError> {
    let target_kind = ProjectLinkTargetKind::parse(row.try_get::<String, _>("target_kind")?)?;
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;
    Ok(ProjectLinkReview {
        project_id: row.try_get("project_id")?,
        target_kind,
        target_id: row.try_get("target_id")?,
        review_state,
        event_id: row.try_get("event_id")?,
        actor_id: row.try_get("actor_id")?,
        reviewed_at: row.try_get("reviewed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_project_reviewed_target(
    row: PgRow,
) -> Result<ProjectReviewedTarget, ProjectLinkReviewError> {
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;

    Ok(ProjectReviewedTarget {
        target_id: row.try_get("target_id")?,
        review_state,
    })
}
