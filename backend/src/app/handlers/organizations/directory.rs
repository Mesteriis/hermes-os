use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore, OrganizationUpdate};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrganizationListResponse {
    items: Vec<Organization>,
}

#[derive(Deserialize)]
pub(crate) struct OrganizationListQuery {
    org_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_organizations(
    State(state): State<AppState>,
    Query(query): Query<OrganizationListQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = OrganizationStore::new(pool)
        .list(query.org_type.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(OrganizationListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrganizationRequest {
    display_name: String,
    org_type: Option<String>,
}

pub(crate) async fn post_organization(
    State(state): State<AppState>,
    Json(req): Json<NewOrganizationRequest>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    let org = OrganizationCommandService::new(pool)
        .create_organization_manual(&req.display_name, req.org_type.as_deref())
        .await?;
    Ok(Json(org))
}

pub(crate) async fn get_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    OrganizationStore::new(pool)
        .get(&org_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(update): Json<OrganizationUpdate>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    let org = OrganizationCommandService::new(pool)
        .update_organization_manual(&org_id, &update)
        .await?;
    Ok(Json(org))
}

#[derive(Deserialize)]
pub(crate) struct OrganizationSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_organization_search(
    State(state): State<AppState>,
    Query(query): Query<OrganizationSearchQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let store = OrganizationStore::new(pool);
    let all = store.list(None, 200).await?;
    let q = query.q.trim().to_lowercase();
    let items: Vec<_> = all
        .into_iter()
        .filter(|o| {
            o.display_name.to_lowercase().contains(&q)
                || o.legal_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                || o.website
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
        })
        .take(query.limit.unwrap_or(20).clamp(1, 100) as usize)
        .collect();
    Ok(Json(OrganizationListResponse { items }))
}

pub(crate) async fn post_organization_archive(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    OrganizationCommandService::new(pool)
        .archive_organization_manual(&org_id)
        .await?;
    Ok(Json(json!({"archived": true})))
}
