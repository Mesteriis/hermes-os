use super::dto::StatusUpdate;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::TaskStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<StatusUpdate>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool).set_status(&id, &r.status).await?;
    Ok(Json(serde_json::json!({"updated":true})))
}
