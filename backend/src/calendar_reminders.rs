use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarReminder {
    pub id: String,
    pub event_id: String,
    pub reminder_type: String,
    pub minutes_before: Option<i32>,
    pub condition_json: Option<Value>,
    pub message: Option<String>,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarReminderStore {
    pool: PgPool,
}

impl CalendarReminderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<CalendarReminder>, ReminderError> {
        let rows = sqlx::query("SELECT id::text, event_id, reminder_type, minutes_before, condition_json, message, is_active, last_triggered_at, created_at, updated_at FROM calendar_reminders WHERE event_id=$1 ORDER BY minutes_before NULLS LAST")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(CalendarReminder {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    reminder_type: r.try_get("reminder_type")?,
                    minutes_before: r.try_get("minutes_before")?,
                    condition_json: r.try_get("condition_json")?,
                    message: r.try_get("message")?,
                    is_active: r.try_get("is_active")?,
                    last_triggered_at: r.try_get("last_triggered_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        event_id: &str,
        reminder_type: &str,
        minutes_before: Option<i32>,
        message: Option<&str>,
    ) -> Result<CalendarReminder, ReminderError> {
        let row = sqlx::query("INSERT INTO calendar_reminders (event_id, reminder_type, minutes_before, message) VALUES ($1,$2,$3,$4) RETURNING id::text, event_id, reminder_type, minutes_before, condition_json, message, is_active, last_triggered_at, created_at, updated_at")
            .bind(event_id).bind(reminder_type).bind(minutes_before).bind(message).fetch_one(&self.pool).await?;
        Ok(CalendarReminder {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            reminder_type: row.try_get("reminder_type")?,
            minutes_before: row.try_get("minutes_before")?,
            condition_json: row.try_get("condition_json")?,
            message: row.try_get("message")?,
            is_active: row.try_get("is_active")?,
            last_triggered_at: row.try_get("last_triggered_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn set_active(&self, id: &str, active: bool) -> Result<(), ReminderError> {
        sqlx::query(
            "UPDATE calendar_reminders SET is_active=$2, updated_at=now() WHERE id=$3::uuid",
        )
        .bind(active)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ReminderError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
