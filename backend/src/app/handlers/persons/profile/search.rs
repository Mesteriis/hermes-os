use super::super::support::*;
use super::models::PersonListResponse;

#[derive(Deserialize)]
pub(crate) struct PersonSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_person_search(
    State(state): State<AppState>,
    Query(query): Query<PersonSearchQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::persons::enrichment::PersonEnrichmentStore,
    >(pool);
    let items = store
        .search_persons(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(PersonListResponse { items }))
}
