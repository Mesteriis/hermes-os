use super::support::*;

// ── Persona Health ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaHealthResponse {
    items: Vec<crate::domains::personas::health::PersonaHealth>,
}

pub(crate) async fn get_persona_health(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<crate::domains::personas::health::PersonaHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<PersonaHealthStore>(pool)
        .get(&persona_id)
        .await
        .map_err(ApiError::from)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn get_personas_health(
    State(state): State<AppState>,
) -> Result<Json<PersonaHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonaHealthStore>(pool)
        .list_health()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaHealthResponse { items }))
}

pub(crate) async fn get_personas_watchlist(
    State(state): State<AppState>,
) -> Result<Json<PersonaHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonaHealthStore>(pool)
        .list_watchlist()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaHealthResponse { items }))
}

pub(crate) async fn post_persona_watchlist_toggle(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let on = crate::domains::personas::service::PersonaCommandService::new(pool)
        .toggle_watchlist_manual(&persona_id)
        .await?;
    Ok(Json(json!({"watchlist": on})))
}
