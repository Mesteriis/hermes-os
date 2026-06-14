use sqlx::Row;
use sqlx::postgres::PgRow;

use super::models::CalendarEvent;

pub(super) fn row_to_event(row: PgRow) -> Result<CalendarEvent, sqlx::Error> {
    Ok(CalendarEvent {
        event_id: row.try_get("event_id")?,
        source_event_id: row.try_get("source_event_id")?,
        account_id: row.try_get("account_id")?,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_at: row.try_get("start_at")?,
        end_at: row.try_get("end_at")?,
        timezone: row.try_get("timezone")?,
        all_day: row.try_get("all_day")?,
        recurrence_rule: row.try_get("recurrence_rule")?,
        status: row.try_get("status")?,
        visibility: row.try_get("visibility")?,
        event_type: row.try_get("event_type")?,
        importance_score: row.try_get("importance_score")?,
        readiness_score: row.try_get("readiness_score")?,
        sync_status: row.try_get("sync_status")?,
        conference_url: row.try_get("conference_url")?,
        conference_provider: row.try_get("conference_provider")?,
        preparation_reminder_minutes: row.try_get("preparation_reminder_minutes")?,
        travel_buffer_minutes: row.try_get("travel_buffer_minutes")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
