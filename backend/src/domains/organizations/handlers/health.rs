use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::health::{OrgHealthStore, OrgRisk, OrgRiskStore};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgRisksResponse {
    items: Vec<OrgRisk>,
}

pub(crate) async fn get_org_risks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRisksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = OrgRiskStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRisksResponse { items }))
}

pub(crate) async fn get_org_health(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let health = OrgHealthStore::new(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&health).unwrap_or_default()))
}

pub(crate) async fn post_org_watchlist_toggle(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let on = OrgHealthStore::new(pool)
        .toggle_watchlist(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"watchlist": on})))
}
