use super::super::*;

pub(crate) async fn get_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack =
        crate::app::api_support::stores::domain_stores::app_store::<EventContextPackStore>(pool)
            .get(&event_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

pub(crate) async fn post_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<ContextPackInput>,
) -> Result<Json<crate::domains::calendar::core::EventContextPack>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack =
        crate::app::api_support::stores::domain_stores::app_store::<EventContextPackStore>(pool)
            .upsert(&event_id, &req)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(pack))
}
