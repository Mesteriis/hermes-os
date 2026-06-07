use super::dto::UpdateRequest;
use crate::app::handlers::{ApiError, AppState};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};
use crate::platform::settings::{ApplicationSetting, ApplicationSettingsStore};
use axum::Json;
use axum::extract::{Path, State};
use serde_json::Value;

const ACTOR: &str = "hermes-frontend";

pub async fn update(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<UpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    ApiAuditLog::new(pool.clone())
        .record(&NewApiAuditRecord::application_setting_set(ACTOR, &key))
        .await?;
    let setting = ApplicationSettingsStore::new(pool)
        .update_setting_value(&key, &Value::String(req.value), ACTOR)
        .await?;
    Ok(Json(setting))
}
