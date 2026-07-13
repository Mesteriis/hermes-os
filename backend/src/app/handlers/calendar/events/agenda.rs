use super::super::*;

pub(crate) async fn get_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda =
        crate::app::api_support::stores::domain_stores::app_store::<EventAgendaStore>(pool)
            .get(&event_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&agenda).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetAgendaRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetAgendaRequest>,
) -> Result<Json<crate::domains::calendar::core::EventAgenda>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let items = req.items;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = CalendarCommandService::new(pool)
        .set_event_agenda_manual(&event_id, items, requested_source)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}
