use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::rules::TaskRuleStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn delete_rule(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskRuleStore::new(pool).delete(&id).await?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
