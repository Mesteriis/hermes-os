use super::super::*;

pub(crate) async fn get_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let checklist = EventChecklistStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&checklist).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetChecklistRequest>,
) -> Result<Json<crate::domains::calendar::core::EventChecklist>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let checklist = EventChecklistStore::new(pool)
        .set(
            &event_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(checklist))
}
