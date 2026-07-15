use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::organization_persona_links::OrganizationPersonaLinkApplicationService;
use crate::domains::organizations::core::aliases::{OrgAliasStore, OrganizationAlias};
use crate::domains::organizations::core::departments::{OrgDepartment, OrgDepartmentStore};
use crate::domains::organizations::core::domains::{OrgDomainStore, OrganizationDomain};
use crate::domains::organizations::core::identity::{OrgIdentityStore, OrganizationIdentity};
use crate::domains::organizations::core::persona_links::{OrgPersonaLink, OrgPersonaLinkStore};
use crate::domains::organizations::core::related::{RelatedOrgStore, RelatedOrganization};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgIdentitiesResponse {
    items: Vec<OrganizationIdentity>,
}

pub(crate) async fn get_org_identities(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgIdentitiesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<OrgIdentityStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgIdentitiesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_org_identity(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgIdentityRequest>,
) -> Result<Json<OrganizationIdentity>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let identity = OrganizationCommandService::new(pool)
        .add_identity_manual(
            &org_id,
            &req.identity_type,
            &req.identity_value,
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

#[derive(Serialize)]
pub(crate) struct OrgAliasesResponse {
    items: Vec<OrganizationAlias>,
}

pub(crate) async fn get_org_aliases(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgAliasesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<OrgAliasStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgAliasesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgAliasRequest {
    name: String,
    alias_type: String,
    source: Option<String>,
}

pub(crate) async fn post_org_alias(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgAliasRequest>,
) -> Result<Json<OrganizationAlias>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let alias = OrganizationCommandService::new(pool)
        .add_alias_manual(&org_id, &req.name, &req.alias_type, requested_source)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(alias))
}

#[derive(Serialize)]
pub(crate) struct OrgDomainsResponse {
    items: Vec<OrganizationDomain>,
}

pub(crate) async fn get_org_domains(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDomainsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<OrgDomainStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgDomainsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgDepartmentsResponse {
    items: Vec<OrgDepartment>,
}

pub(crate) async fn get_org_departments(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDepartmentsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<OrgDepartmentStore>(pool)
            .list(&org_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(OrgDepartmentsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgDepartmentRequest {
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
}

pub(crate) async fn post_org_department(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgDepartmentRequest>,
) -> Result<Json<OrgDepartment>, ApiError> {
    let pool = database_pool(&state)?;
    let dept = OrganizationCommandService::new(pool)
        .add_department_manual(
            &org_id,
            &req.name,
            req.description.as_deref(),
            req.parent_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dept))
}

#[derive(Serialize)]
pub(crate) struct OrgPersonaLinksResponse {
    items: Vec<OrgPersonaLink>,
}

pub(crate) async fn get_org_persona_links(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPersonaLinksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<OrgPersonaLinkStore>(pool)
            .list_by_org(&org_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(OrgPersonaLinksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct LinkOrgPersonaRequest {
    #[serde(alias = "person_id")]
    persona_id: String,
    role: Option<String>,
    department: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_org_persona_link(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<LinkOrgPersonaRequest>,
) -> Result<Json<OrgPersonaLink>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let link = OrganizationPersonaLinkApplicationService::new(pool)
        .link_persona_manual(
            &org_id,
            &req.persona_id,
            req.role.as_deref(),
            req.department.as_deref(),
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(link))
}

#[derive(Serialize)]
pub(crate) struct OrgRelatedResponse {
    items: Vec<RelatedOrganization>,
}

pub(crate) async fn get_org_related(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRelatedResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::stores::domain_stores::app_store::<RelatedOrgStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRelatedResponse { items }))
}
