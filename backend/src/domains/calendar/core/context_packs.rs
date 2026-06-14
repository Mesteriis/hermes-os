use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventContextPack {
    pub id: String,
    pub event_id: String,
    pub summary: Option<String>,
    pub participants_summary: Option<String>,
    pub documents: Value,
    pub tasks: Value,
    pub open_questions: Value,
    pub risks: Value,
    pub suggested_agenda: Value,
    pub suggested_actions: Value,
    pub generated_at: DateTime<Utc>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventContextPackStore {
    pool: PgPool,
}

impl EventContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventContextPack>, CalendarCoreError> {
        let row = sqlx::query("SELECT id::text, event_id, summary, participants_summary, documents, tasks, open_questions, risks, suggested_agenda, suggested_actions, generated_at, model, created_at, updated_at FROM event_context_packs WHERE event_id=$1 ORDER BY generated_at DESC LIMIT 1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(EventContextPack {
                id: r.try_get("id")?,
                event_id: r.try_get("event_id")?,
                summary: r.try_get("summary")?,
                participants_summary: r.try_get("participants_summary")?,
                documents: r.try_get("documents")?,
                tasks: r.try_get("tasks")?,
                open_questions: r.try_get("open_questions")?,
                risks: r.try_get("risks")?,
                suggested_agenda: r.try_get("suggested_agenda")?,
                suggested_actions: r.try_get("suggested_actions")?,
                generated_at: r.try_get("generated_at")?,
                model: r.try_get("model")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn upsert(
        &self,
        event_id: &str,
        pack: &ContextPackInput,
    ) -> Result<EventContextPack, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_context_packs (event_id, summary, participants_summary, documents, tasks, open_questions, risks, suggested_agenda, suggested_actions, model) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT DO NOTHING RETURNING id::text, event_id, summary, participants_summary, documents, tasks, open_questions, risks, suggested_agenda, suggested_actions, generated_at, model, created_at, updated_at")
            .bind(event_id).bind(pack.summary.as_deref()).bind(pack.participants_summary.as_deref())
            .bind(&pack.documents).bind(&pack.tasks).bind(&pack.open_questions)
            .bind(&pack.risks).bind(&pack.suggested_agenda).bind(&pack.suggested_actions)
            .bind(pack.model.as_deref()).fetch_one(&self.pool).await?;
        Ok(EventContextPack {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            summary: row.try_get("summary")?,
            participants_summary: row.try_get("participants_summary")?,
            documents: row.try_get("documents")?,
            tasks: row.try_get("tasks")?,
            open_questions: row.try_get("open_questions")?,
            risks: row.try_get("risks")?,
            suggested_agenda: row.try_get("suggested_agenda")?,
            suggested_actions: row.try_get("suggested_actions")?,
            generated_at: row.try_get("generated_at")?,
            model: row.try_get("model")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ContextPackInput {
    pub summary: Option<String>,
    pub participants_summary: Option<String>,
    pub documents: Value,
    pub tasks: Value,
    pub open_questions: Value,
    pub risks: Value,
    pub suggested_agenda: Value,
    pub suggested_actions: Value,
    pub model: Option<String>,
}
