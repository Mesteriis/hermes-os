use super::super::support::*;
use super::models::EnrichedPersonaListResponse;

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
    let store = crate::app::api_support::app_store::<
        crate::domains::personas::enrichment::PersonaEnrichmentStore,
    >(pool);
    let items = store
        .search_personas(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(EnrichedPersonaListResponse { items }))
}
