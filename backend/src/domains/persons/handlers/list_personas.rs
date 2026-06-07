use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::core::PersonPersonaStore;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub async fn list_personas(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonPersonaStore::new(pool).list_by_person(&id).await?)
            .unwrap_or_default(),
    ))
}
