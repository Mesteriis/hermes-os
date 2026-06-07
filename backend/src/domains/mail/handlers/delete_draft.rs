use crate::app::handlers::{ApiError, AppState};
use crate::domains::mail::drafts::EmailDraftStore;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub async fn delete_draft(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = EmailDraftStore::new(pool);
    let result = store.delete(&id).await?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}
