use axum::Json;
use axum::extract::{Path, State};
use serde_json::Value;

use crate::app::{ApiError, AppState};
use crate::domains::organizations::investigator::OrganizationInvestigator;

use super::support::database_pool;

pub(crate) async fn get_org_dossier(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let dossier = OrganizationInvestigator::new(pool)
        .dossier(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

pub(crate) async fn get_org_brief(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let brief = OrganizationInvestigator::new(pool)
        .brief(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&brief).unwrap_or_default()))
}

pub(crate) async fn get_org_context_pack(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let pack = OrganizationInvestigator::new(pool)
        .context_pack(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}
