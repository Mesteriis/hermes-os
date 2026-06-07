use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::expertise::PersonExpertiseStore;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub async fn expertise(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonExpertiseStore::new(pool).list(&id).await?).unwrap_or_default(),
    ))
}
