use super::*;

// ── Calendar Import/Export ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarImportRequest {
    ics_data: Option<String>,
    events: Option<Value>,
}

pub(crate) async fn post_calendar_import(
    State(state): State<AppState>,
    Json(req): Json<CalendarImportRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let ics_data_received = req
        .ics_data
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let mut imported = 0;
    if let Some(events) = req.events
        && let Some(arr) = events.as_array()
    {
        for evt in arr {
            let title = evt
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Imported Event");
            let start = evt
                .get("start_at")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or(Utc::now());
            let end = evt
                .get("end_at")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or(start);
            let source_event_id = evt
                .get("source_event_id")
                .and_then(|v| v.as_str())
                .map(ToOwned::to_owned);
            let _ =
                crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(
                    pool.clone(),
                )
                .create_file_import(
                    &NewCalendarEvent {
                        source_event_id,
                        title: title.to_string(),
                        start_at: start,
                        end_at: end,
                        ..Default::default()
                    },
                    &format!("calendar-import://event/{imported}"),
                )
                .await;
            imported += 1;
        }
    }
    Ok(Json(
        json!({"imported": imported, "ics_data_received": ics_data_received}),
    ))
}

pub(crate) async fn post_calendar_sync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .trigger_calendar_sync_manual(&account_id)
        .await?;
    Ok(Json(
        json!({"sync_triggered": true, "note": "Provider sync is deferred to future implementation"}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct EventExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_event_export(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event =
        crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
            .get(&event_id)
            .await?
            .ok_or(ApiError::NotFound)?;
    let fmt = query.format.as_deref().unwrap_or("json");
    match fmt {
        "ics" => {
            let ics = export_event_ics(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.format("%Y%m%dT%H%M%S").to_string(),
                &event.end_at.format("%Y%m%dT%H%M%S").to_string(),
                event.timezone.as_deref(),
            );
            Ok(Json(json!({"format": "ics", "content": ics})))
        }
        "md" => {
            let md = export_event_md(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.to_rfc3339(),
                &event.end_at.to_rfc3339(),
                &[],
            );
            Ok(Json(json!({"format": "markdown", "content": md})))
        }
        _ => Ok(Json(serde_json::to_value(&event).unwrap_or_default())),
    }
}
