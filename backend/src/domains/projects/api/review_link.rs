use super::dto::{LinkReviewRequest, LinkReviewResponse};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewState, ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};
use axum::Json;
use axum::extract::{Path, State};

const ACTOR: &str = "hermes-frontend";

pub async fn review_link(
    State(state): State<AppState>,
    Path(pid): Path<String>,
    req: Json<LinkReviewRequest>,
) -> Result<Json<LinkReviewResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let kind: ProjectLinkTargetKind = match req.target_kind.as_str() {
        "message" => ProjectLinkTargetKind::Message,
        "document" => ProjectLinkTargetKind::Document,
        _ => return Err(ApiError::InvalidProjectLinkReview("invalid target_kind")),
    };
    let review_state: ProjectLinkReviewState = match req.review_state.as_str() {
        "user_confirmed" => ProjectLinkReviewState::UserConfirmed,
        "user_rejected" => ProjectLinkReviewState::UserRejected,
        _ => return Err(ApiError::InvalidProjectLinkReview("invalid review_state")),
    };
    ApiAuditLog::new(pool.clone())
        .record(&NewApiAuditRecord::project_link_review_set(
            ACTOR,
            &pid,
            kind.as_str(),
            &req.target_id,
        ))
        .await?;
    let result = ProjectLinkReviewStore::new(pool)
        .set_review_state(
            &crate::domains::projects::link_reviews::ProjectLinkReviewCommand {
                command_id: format!("review-{}-{}", pid, req.target_id.clone()),
                actor_id: ACTOR.into(),
                project_id: pid,
                target_kind: kind,
                target_id: req.target_id.clone(),
                review_state,
            },
        )
        .await?;
    Ok(Json(LinkReviewResponse {
        project_id: result.project_id,
        target_kind: result.target_kind.as_str().into(),
        target_id: result.target_id,
        review_state: result.review_state.as_str().into(),
    }))
}
