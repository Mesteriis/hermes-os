use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::OrganizationStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn archive(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrganizationStore::new(pool).archive(&id).await?;
    Ok(Json(serde_json::json!({"archived":true})))
}
