use super::dto::NewIdentity;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgIdentityStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn create_identity(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
    Json(r): Json<NewIdentity>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = OrgIdentityStore::new(pool)
        .upsert(
            &org_id,
            &r.identity_type,
            &r.identity_value,
            r.source.as_deref().unwrap_or("manual"),
        )
        .await?;
    Ok(Json(serde_json::to_value(item).unwrap_or_default()))
}
