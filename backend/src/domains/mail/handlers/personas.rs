use crate::app::handlers::{ApiError, AppState};
use crate::domains::mail::personas::EmailPersonaStore;
use axum::Json;
use axum::extract::State;

pub(crate) async fn list_personas(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EmailPersonaStore::new(pool).list().await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
