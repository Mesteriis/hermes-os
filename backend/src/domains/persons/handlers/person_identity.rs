use axum::extract::{Path, State}; use axum::Json;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::identity::PersonIdentityStore;

pub(crate) async fn person_identity(State(state): State<AppState>, Path(person_id): Path<String>) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    Ok(Json(serde_json::to_value(PersonIdentityStore::new(pool).person_identity(&person_id).await?).unwrap_or_default()))
}
