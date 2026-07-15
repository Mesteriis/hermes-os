use super::*;

// ── Event Reminders ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRemindersResponse {
    items: Vec<crate::domains::calendar::reminders::CalendarReminder>,
}

pub(crate) async fn get_event_reminders(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRemindersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<CalendarReminderStore>(pool)
            .list(&event_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(EventRemindersResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewReminderRequest {
    reminder_type: String,
    minutes_before: Option<i32>,
    message: Option<String>,
}

pub(crate) async fn post_event_reminder(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewReminderRequest>,
) -> Result<Json<crate::domains::calendar::reminders::CalendarReminder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let r = CalendarCommandService::new(pool)
        .create_event_reminder_manual(
            &event_id,
            &req.reminder_type,
            req.minutes_before,
            req.message.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(r))
}

#[derive(Deserialize)]
pub(crate) struct ToggleReminderRequest {
    active: bool,
}

pub(crate) async fn post_event_reminder_toggle(
    State(state): State<AppState>,
    Path((event_id, reminder_id)): Path<(String, String)>,
    Json(req): Json<ToggleReminderRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .toggle_event_reminder_manual(&event_id, &reminder_id, req.active)
        .await?;
    Ok(Json(json!({"toggled": true, "active": req.active})))
}
