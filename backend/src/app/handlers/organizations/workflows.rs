use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::workflows::{
    OrgPlaybook, OrgPlaybookStore, OrgPortal, OrgPortalStore, OrgProcedure, OrgProcedureStore,
    OrgTemplate, OrgTemplateStore, OrgTimelineEvent, OrgTimelineStore,
};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgTimelineResponse {
    items: Vec<OrgTimelineEvent>,
}

#[derive(Deserialize)]
pub(crate) struct OrgTimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn get_org_timeline(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Query(query): Query<OrgTimelineQuery>,
) -> Result<Json<OrgTimelineResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgTimelineStore>(pool)
        .list(&org_id, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTimelineResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgPortalsResponse {
    items: Vec<OrgPortal>,
}

pub(crate) async fn get_org_portals(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPortalsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgPortalStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPortalsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgProceduresResponse {
    items: Vec<OrgProcedure>,
}

pub(crate) async fn get_org_procedures(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProceduresResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgProcedureStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProceduresResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgPlaybooksResponse {
    items: Vec<OrgPlaybook>,
}

pub(crate) async fn get_org_playbooks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPlaybooksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgPlaybookStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPlaybooksResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgTemplatesResponse {
    items: Vec<OrgTemplate>,
}

pub(crate) async fn get_org_templates(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgTemplatesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgTemplateStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTemplatesResponse { items }))
}
