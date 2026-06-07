use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::core::ExternalTaskIdentityStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn external(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(ExternalTaskIdentityStore::new(pool).list(&id).await?)
            .unwrap_or_default(),
    ))
}
