use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::Value;

use crate::app::{ApiError, AppState};
use crate::domains::organizations::finance::{
    OrgCompliance, OrgComplianceStore, OrgContract, OrgContractStore, OrgFinancialStore,
    OrgProduct, OrgProductStore, OrgService, OrgServiceStore,
};

use super::support::database_pool;

pub(crate) async fn get_org_financial(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let info = crate::app::api_support::app_store::<OrgFinancialStore>(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&info).unwrap_or_default()))
}

#[derive(Serialize)]
pub(crate) struct OrgContractsResponse {
    items: Vec<OrgContract>,
}

pub(crate) async fn get_org_contracts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContractsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgContractStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContractsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgComplianceResponse {
    items: Vec<OrgCompliance>,
}

pub(crate) async fn get_org_compliance(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgComplianceResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgComplianceStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgComplianceResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgServicesResponse {
    items: Vec<OrgService>,
}

pub(crate) async fn get_org_services(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgServicesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgServiceStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgServicesResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgProductsResponse {
    items: Vec<OrgProduct>,
}

pub(crate) async fn get_org_products(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProductsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgProductStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProductsResponse { items }))
}
