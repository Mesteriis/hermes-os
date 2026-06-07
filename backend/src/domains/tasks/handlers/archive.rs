use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::TaskStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn archive(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool).archive(&id).await?;
    Ok(Json(serde_json::json!({"archived":true})))
}
