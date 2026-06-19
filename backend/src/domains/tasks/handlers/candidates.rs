use axum::Json;
use axum::extract::{Path, RawQuery, State};
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TaskCandidateListResponse, TaskCandidateReviewApiRequest, TaskCandidateReviewApiResponse,
    api_audit_log, observation_store, parse_task_candidates_query, task_candidate_store,
};
use crate::platform::audit::NewApiAuditRecord;
pub(crate) async fn get_task_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(TaskCandidateListResponse { items }))
}

pub(crate) async fn put_task_candidate_review(
    State(state): State<AppState>,
    Path(task_candidate_id): Path<String>,
    Json(request): Json<TaskCandidateReviewApiRequest>,
) -> Result<Json<TaskCandidateReviewApiResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let command = request.into_command(task_candidate_id, actor_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::task_candidate_review_set(
            &command.actor_id,
            &command.task_candidate_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = crate::domains::tasks::candidates::TaskCandidateReviewService::new(pool)
        .review_manual(&command)
        .await?;

    Ok(Json(result.into()))
}
