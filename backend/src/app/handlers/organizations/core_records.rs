use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::domains::organizations::core::{
    OrgAliasStore, OrgContactLink, OrgContactLinkStore, OrgDepartment, OrgDepartmentStore,
    OrgDomainStore, OrgIdentityStore, OrganizationAlias, OrganizationDomain, OrganizationIdentity,
    RelatedOrgStore, RelatedOrganization,
};
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
    let items = crate::app::api_support::app_store::<OrgIdentityStore>(pool)
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
    let items = crate::app::api_support::app_store::<OrgAliasStore>(pool)
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
    let items = crate::app::api_support::app_store::<OrgDomainStore>(pool)
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
    let items = crate::app::api_support::app_store::<OrgDepartmentStore>(pool)
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
pub(crate) struct OrgContactsResponse {
    items: Vec<OrgContactLink>,
}

pub(crate) async fn get_org_contacts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContactsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgContactLinkStore>(pool)
        .list_by_org(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct LinkOrgContactRequest {
    person_id: String,
    role: Option<String>,
    department: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_org_contact_link(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<LinkOrgContactRequest>,
) -> Result<Json<OrgContactLink>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let link = OrganizationCommandService::new(pool)
        .link_contact_manual(
            &org_id,
            &req.person_id,
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
    let items = crate::app::api_support::app_store::<RelatedOrgStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRelatedResponse { items }))
}
