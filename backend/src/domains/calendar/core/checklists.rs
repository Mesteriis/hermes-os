use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;
use super::link_calendar_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventChecklist {
    pub id: String,
    pub event_id: String,
    pub items: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventChecklistStore {
    pool: PgPool,
}

impl EventChecklistStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventChecklist>, CalendarCoreError> {
        let row = sqlx::query("SELECT id::text, event_id, items, source, created_at, updated_at FROM event_checklists WHERE event_id=$1 ORDER BY created_at DESC LIMIT 1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(EventChecklist {
                id: r.try_get("id")?,
                event_id: r.try_get("event_id")?,
                items: r.try_get("items")?,
                source: r.try_get("source")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn set(
        &self,
        event_id: &str,
        items: Value,
        source: &str,
    ) -> Result<EventChecklist, CalendarCoreError> {
        self.set_with_observation(event_id, items, source, None)
            .await
    }

    pub async fn set_with_observation(
        &self,
        event_id: &str,
        items: Value,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<EventChecklist, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_checklists (event_id, items, source) VALUES ($1,$2,$3) RETURNING id::text, event_id, items, source, created_at, updated_at")
            .bind(event_id).bind(&items).bind(source).fetch_one(&self.pool).await?;
        let checklist = EventChecklist {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "event_checklist",
                checklist.id.clone(),
                Some(serde_json::json!({
                    "event_id": event_id,
                })),
            )
            .await?;
        }
        Ok(checklist)
    }
}
