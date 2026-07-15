use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::models::EnrichedPersonaListResponse;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;

#[derive(Deserialize)]
pub(crate) struct PersonaSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_persona_search(
    State(state): State<AppState>,
    Query(query): Query<PersonaSearchQuery>,
) -> Result<Json<EnrichedPersonaListResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::personas::enrichment::store::PersonaEnrichmentStore,
    >(pool);
    let items = store
        .search_personas(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(EnrichedPersonaListResponse { items }))
}
