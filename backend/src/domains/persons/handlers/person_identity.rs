use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::identity::PersonIdentityStore;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub(crate) async fn person_identity(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonIdentityStore::new(pool).person_identity(&id).await?)
            .unwrap_or_default(),
    ))
}
