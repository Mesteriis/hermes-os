use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::meetings::MeetingOutcomeStore;
use axum::Json;
use axum::extract::{Path, State};

pub(crate) async fn follow_up(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let status = MeetingOutcomeStore::new(pool)
        .follow_up_status(&event_id)
        .await?;
    Ok(Json(status))
}
