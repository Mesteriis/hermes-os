use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::trust::PersonPromiseStore;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub async fn promises(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonPromiseStore::new(pool).list(&id).await?).unwrap_or_default(),
    ))
}
