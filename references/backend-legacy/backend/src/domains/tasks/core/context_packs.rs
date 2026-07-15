use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;
use crate::engines::context_packs::{
    models::{
        ContextPack, ContextPackKind, ContextPackSourceKind, NewContextPack, NewContextPackSource,
    },
    store::ContextPackStore,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskContextPack {
    pub id: String,
    pub task_id: String,
    pub summary: Option<String>,
    pub source_summary: Option<String>,
    pub open_questions: Value,
    pub blockers: Value,
    pub risks: Value,
    pub suggested_next_action: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskContextPackStore {
    pool: PgPool,
}

impl TaskContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<TaskContextPack>, TaskCoreError> {
        ContextPackStore::new(self.pool.clone())
            .get(ContextPackKind::Task, task_id)
            .await?
            .map(|pack| task_context_pack_from_engine(pack, task_id))
            .transpose()
    }

    pub async fn upsert(
        &self,
        task_id: &str,
        summary: Option<&str>,
        questions: Value,
        blockers: Value,
        risks: Value,
        next_action: Option<&str>,
    ) -> Result<TaskContextPack, TaskCoreError> {
        let stored = ContextPackStore::new(self.pool.clone())
            .upsert_with_sources(
                &NewContextPack::new(
                    ContextPackKind::Task,
                    task_id,
                    json!({
                        "summary": summary,
                        "source_summary": summary,
                        "open_questions": questions,
                        "blockers": blockers,
                        "risks": risks,
                        "suggested_next_action": next_action,
                    }),
                )
                .metadata(json!({
                    "owner": "domains.tasks.core.context_packs",
                })),
                &[NewContextPackSource::new(ContextPackSourceKind::Task, task_id).role("subject")],
            )
            .await?;
        task_context_pack_from_engine(stored, task_id)
    }
}

fn task_context_pack_from_engine(
    pack: ContextPack,
    task_id: &str,
) -> Result<TaskContextPack, TaskCoreError> {
    let content = &pack.content;
    Ok(TaskContextPack {
        id: pack.context_pack_id,
        task_id: task_id.to_owned(),
        summary: optional_string(content, "summary"),
        source_summary: optional_string(content, "source_summary"),
        open_questions: content
            .get("open_questions")
            .cloned()
            .unwrap_or_else(|| json!([])),
        blockers: content
            .get("blockers")
            .cloned()
            .unwrap_or_else(|| json!([])),
        risks: content.get("risks").cloned().unwrap_or_else(|| json!([])),
        suggested_next_action: optional_string(content, "suggested_next_action"),
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
