use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::enrichment::{OrgEnrichmentResult, OrgEnrichmentStore};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgEnrichmentResponse {
    items: Vec<OrgEnrichmentResult>,
}

pub(crate) async fn get_org_enrichment(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgEnrichmentResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = OrgEnrichmentStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgEnrichmentResponse { items }))
}

pub(crate) async fn post_org_enrich_apply(
    State(state): State<AppState>,
    Path((_org_id, rid)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    OrgEnrichmentStore::new(pool)
        .apply(&rid)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"applied": true})))
}
