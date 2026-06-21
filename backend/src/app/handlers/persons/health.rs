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
    crate::app::api_support::app_store::<PersonHealthStore>(pool)
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
    let items = crate::app::api_support::app_store::<PersonHealthStore>(pool)
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
    let items = crate::app::api_support::app_store::<PersonHealthStore>(pool)
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
    let on = crate::domains::persons::service::PersonCommandService::new(pool)
        .toggle_watchlist_manual(&person_id)
        .await?;
    Ok(Json(json!({"watchlist": on})))
}
