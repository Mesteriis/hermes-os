use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;
use crate::engines::context_packs::{
    models::{
        ContextPack, ContextPackKind, ContextPackSourceKind, NewContextPack, NewContextPackSource,
    },
    store::ContextPackStore,
};

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
        ContextPackStore::new(self.pool.clone())
            .get(ContextPackKind::Calendar, event_id)
            .await?
            .map(|pack| event_context_pack_from_engine(pack, event_id))
            .transpose()
    }

    pub async fn upsert(
        &self,
        event_id: &str,
        pack: &ContextPackInput,
    ) -> Result<EventContextPack, CalendarCoreError> {
        let stored = ContextPackStore::new(self.pool.clone())
            .upsert_with_sources(
                &NewContextPack::new(
                    ContextPackKind::Calendar,
                    event_id,
                    json!({
                        "summary": pack.summary,
                        "participants_summary": pack.participants_summary,
                        "documents": pack.documents,
                        "tasks": pack.tasks,
                        "open_questions": pack.open_questions,
                        "risks": pack.risks,
                        "suggested_agenda": pack.suggested_agenda,
                        "suggested_actions": pack.suggested_actions,
                    }),
                )
                .metadata(json!({
                    "model": pack.model,
                    "owner": "domains.calendar.core.context_packs",
                })),
                &[
                    NewContextPackSource::new(ContextPackSourceKind::CalendarEvent, event_id)
                        .role("subject"),
                ],
            )
            .await?;
        event_context_pack_from_engine(stored, event_id)
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

fn event_context_pack_from_engine(
    pack: ContextPack,
    event_id: &str,
) -> Result<EventContextPack, CalendarCoreError> {
    let content = &pack.content;
    Ok(EventContextPack {
        id: pack.context_pack_id,
        event_id: event_id.to_owned(),
        summary: optional_string(content, "summary"),
        participants_summary: optional_string(content, "participants_summary"),
        documents: content
            .get("documents")
            .cloned()
            .unwrap_or_else(|| json!([])),
        tasks: content.get("tasks").cloned().unwrap_or_else(|| json!([])),
        open_questions: content
            .get("open_questions")
            .cloned()
            .unwrap_or_else(|| json!([])),
        risks: content.get("risks").cloned().unwrap_or_else(|| json!([])),
        suggested_agenda: content
            .get("suggested_agenda")
            .cloned()
            .unwrap_or_else(|| json!([])),
        suggested_actions: content
            .get("suggested_actions")
            .cloned()
            .unwrap_or_else(|| json!([])),
        generated_at: pack.built_at,
        model: optional_string(&pack.metadata, "model"),
        created_at: pack.built_at,
        updated_at: pack.updated_at,
    })
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}
