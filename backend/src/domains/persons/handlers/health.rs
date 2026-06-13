use super::support::*;

// ── Person Health ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonHealthResponse {
    items: Vec<crate::domains::persons::health::PersonHealth>,
}

pub(crate) async fn get_person_health(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::health::PersonHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    PersonHealthStore::new(pool)
        .get(&person_id)
        .await
        .map_err(ApiError::from)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn get_persons_health(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonHealthStore::new(pool)
        .list_health()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn get_persons_watchlist(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonHealthStore::new(pool)
        .list_watchlist()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn post_person_watchlist_toggle(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let on = PersonHealthStore::new(pool)
        .toggle_watchlist(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"watchlist": on})))
}
