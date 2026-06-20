use super::super::support::*;
use super::models::PersonListResponse;

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}

pub(crate) async fn get_persons(
    State(state): State<AppState>,
    Query(query): Query<PersonListQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .list_enriched(
            query.favorites_only.unwrap_or(false),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(PersonListResponse { items }))
}

pub(crate) async fn get_person(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::enrichment::EnrichedPerson>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    match store.get_enriched(&person_id).await? {
        Some(person) => Ok(Json(person)),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}
