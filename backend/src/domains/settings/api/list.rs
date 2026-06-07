use super::dto::SettingsResponse;
use crate::app::handlers::{ApiError, AppState};
use crate::platform::settings::ApplicationSettingsStore;
use axum::Json;
use axum::extract::State;

pub(crate) async fn list(
    State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = ApplicationSettingsStore::new(pool).list_settings().await?;
    Ok(Json(SettingsResponse { items }))
}
