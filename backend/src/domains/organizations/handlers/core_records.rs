use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::core::{
    OrgAliasStore, OrgContactLink, OrgContactLinkStore, OrgDepartment, OrgDepartmentStore,
    OrgDomainStore, OrgIdentityStore, OrganizationAlias, OrganizationDomain, OrganizationIdentity,
    RelatedOrgStore, RelatedOrganization,
};

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
    let items = OrgIdentityStore::new(pool)
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
    let pool = database_pool(&state)?;
    let identity = OrgIdentityStore::new(pool)
        .upsert(
            &org_id,
            &req.identity_type,
            &req.identity_value,
            req.source.as_deref().unwrap_or("manual"),
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
    let items = OrgAliasStore::new(pool)
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
    let pool = database_pool(&state)?;
    let alias = OrgAliasStore::new(pool)
        .add(
            &org_id,
            &req.name,
            &req.alias_type,
            req.source.as_deref().unwrap_or("manual"),
        )
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
    let items = OrgDomainStore::new(pool)
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
    let items = OrgDepartmentStore::new(pool)
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
    let dept = OrgDepartmentStore::new(pool)
        .add(
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
    let items = OrgContactLinkStore::new(pool)
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
}

pub(crate) async fn post_org_contact_link(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<LinkOrgContactRequest>,
) -> Result<Json<OrgContactLink>, ApiError> {
    let pool = database_pool(&state)?;
    let link = OrgContactLinkStore::new(pool)
        .link(
            &org_id,
            &req.person_id,
            req.role.as_deref(),
            req.department.as_deref(),
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
    let items = RelatedOrgStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRelatedResponse { items }))
}
