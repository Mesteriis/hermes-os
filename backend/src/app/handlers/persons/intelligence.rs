use super::support::*;
// ── Person Enrichment ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EnrichmentResultsResponse {
    items: Vec<crate::domains::persons::enrichment_engine::EnrichmentResult>,
}

pub(crate) async fn get_person_enrichment(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<EnrichmentResultsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EnrichmentResultStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EnrichmentResultsResponse { items }))
}

pub(crate) async fn post_person_enrichment_apply(
    State(state): State<AppState>,
    Path((person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::persons::service::PersonCommandService::new(pool)
        .apply_enrichment_manual(&person_id, &result_id)
        .await?;
    Ok(Json(json!({"applied": true})))
}

pub(crate) async fn post_person_enrichment_reject(
    State(state): State<AppState>,
    Path((person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::persons::service::PersonCommandService::new(pool)
        .reject_enrichment_manual(&person_id, &result_id)
        .await?;
    Ok(Json(json!({"rejected": true})))
}

// ── Person Expertise ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonExpertiseResponse {
    items: Vec<crate::domains::persons::expertise::PersonExpertise>,
}

pub(crate) async fn get_person_expertise(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonExpertiseStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct ExpertiseSearchQuery {
    skill: String,
    limit: Option<i64>,
}

pub(crate) async fn get_person_expertise_search(
    State(state): State<AppState>,
    Query(query): Query<ExpertiseSearchQuery>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonExpertiseStore::new(pool)
        .search_by_skill(&query.skill, query.limit.unwrap_or(20))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

// ── Person Promises ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPromisesResponse {
    items: Vec<crate::domains::persons::trust::PersonPromise>,
}

pub(crate) async fn get_person_promises(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPromisesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonPromiseStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPromisesResponse { items }))
}

// ── Person Risks ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRisksResponse {
    items: Vec<crate::domains::persons::trust::PersonRisk>,
}

pub(crate) async fn get_person_risks(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRisksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonRiskStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRisksResponse { items }))
}
