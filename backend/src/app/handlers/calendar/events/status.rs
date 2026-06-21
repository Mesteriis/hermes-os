use super::super::*;

#[derive(Deserialize)]
pub(crate) struct RescheduleRequest {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

pub(crate) async fn post_calendar_event_reschedule(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .reschedule_manual(&event_id, req.start_at, req.end_at)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn post_calendar_event_cancel(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .set_status_manual(
            &event_id,
            "cancelled",
            "calendar_api.post_calendar_event_cancel",
        )
        .await?;
    Ok(Json(json!({"cancelled": true})))
}
