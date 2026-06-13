use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
    RelationshipStoreError,
};

// ── TaskProviderAccount ───────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskProviderAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub credentials_reference: Option<String>,
    pub sync_mode: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskProviderStore {
    pool: PgPool,
}
impl TaskProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self) -> Result<Vec<TaskProviderAccount>, TaskCoreError> {
        let rows = sqlx::query("SELECT account_id, provider, account_name, credentials_reference, sync_mode, capabilities, created_at, updated_at FROM task_provider_accounts ORDER BY provider, account_name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskProviderAccount {
                    account_id: r.try_get("account_id")?,
                    provider: r.try_get("provider")?,
                    account_name: r.try_get("account_name")?,
                    credentials_reference: r.try_get("credentials_reference")?,
                    sync_mode: r.try_get("sync_mode")?,
                    capabilities: r.try_get("capabilities")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let account_id = format!("tprov:v1:{ts:x}");
        let row = sqlx::query("INSERT INTO task_provider_accounts (account_id, provider, account_name) VALUES ($1,$2,$3) RETURNING account_id, provider, account_name, credentials_reference, sync_mode, capabilities, created_at, updated_at")
            .bind(&account_id).bind(provider).bind(account_name).fetch_one(&self.pool).await?;
        Ok(TaskProviderAccount {
            account_id: row.try_get("account_id")?,
            provider: row.try_get("provider")?,
            account_name: row.try_get("account_name")?,
            credentials_reference: row.try_get("credentials_reference")?,
            sync_mode: row.try_get("sync_mode")?,
            capabilities: row.try_get("capabilities")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── ExternalTaskIdentity ──────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalTaskIdentity {
    pub id: String,
    pub task_id: String,
    pub provider: String,
    pub account_id: Option<String>,
    pub external_project_id: Option<String>,
    pub external_task_id: Option<String>,
    pub external_url: Option<String>,
    pub external_status: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ExternalTaskIdentityStore {
    pool: PgPool,
}
impl ExternalTaskIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, task_id: &str) -> Result<Vec<ExternalTaskIdentity>, TaskCoreError> {
        let rows = sqlx::query("SELECT id::text, task_id, provider, account_id, external_project_id, external_task_id, external_url, external_status, sync_status, last_synced_at, created_at, updated_at FROM external_task_identities WHERE task_id=$1")
            .bind(task_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(ExternalTaskIdentity {
                    id: r.try_get("id")?,
                    task_id: r.try_get("task_id")?,
                    provider: r.try_get("provider")?,
                    account_id: r.try_get("account_id")?,
                    external_project_id: r.try_get("external_project_id")?,
                    external_task_id: r.try_get("external_task_id")?,
                    external_url: r.try_get("external_url")?,
                    external_status: r.try_get("external_status")?,
                    sync_status: r.try_get("sync_status")?,
                    last_synced_at: r.try_get("last_synced_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

// ── TaskContextPack ───────────────────────────────────────────────────────

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
        let row = sqlx::query("SELECT id::text, task_id, summary, source_summary, open_questions, blockers, risks, suggested_next_action, generated_at, model, created_at, updated_at FROM task_context_packs WHERE task_id=$1 ORDER BY generated_at DESC LIMIT 1")
            .bind(task_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(TaskContextPack {
                id: r.try_get("id")?,
                task_id: r.try_get("task_id")?,
                summary: r.try_get("summary")?,
                source_summary: r.try_get("source_summary")?,
                open_questions: r.try_get("open_questions")?,
                blockers: r.try_get("blockers")?,
                risks: r.try_get("risks")?,
                suggested_next_action: r.try_get("suggested_next_action")?,
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
        task_id: &str,
        summary: Option<&str>,
        questions: Value,
        blockers: Value,
        risks: Value,
        next_action: Option<&str>,
    ) -> Result<TaskContextPack, TaskCoreError> {
        let row = sqlx::query("INSERT INTO task_context_packs (task_id, summary, open_questions, blockers, risks, suggested_next_action) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, task_id, summary, source_summary, open_questions, blockers, risks, suggested_next_action, generated_at, model, created_at, updated_at")
            .bind(task_id).bind(summary).bind(&questions).bind(&blockers).bind(&risks).bind(next_action)
            .fetch_one(&self.pool).await?;
        Ok(TaskContextPack {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            summary: row.try_get("summary")?,
            source_summary: row.try_get("source_summary")?,
            open_questions: row.try_get("open_questions")?,
            blockers: row.try_get("blockers")?,
            risks: row.try_get("risks")?,
            suggested_next_action: row.try_get("suggested_next_action")?,
            generated_at: row.try_get("generated_at")?,
            model: row.try_get("model")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── TaskEvidence ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskEvidence {
    pub id: String,
    pub task_id: String,
    pub source_type: String,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskEvidenceStore {
    pool: PgPool,
}
impl TaskEvidenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskEvidence>, TaskCoreError> {
        let rows = sqlx::query("SELECT id::text, task_id, source_type, source_id, quote, confidence::float8 AS confidence, created_at FROM task_evidence WHERE task_id=$1 ORDER BY created_at DESC")
            .bind(task_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskEvidence {
                    id: r.try_get("id")?,
                    task_id: r.try_get("task_id")?,
                    source_type: r.try_get("source_type")?,
                    source_id: r.try_get("source_id")?,
                    quote: r.try_get("quote")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        task_id: &str,
        source_type: &str,
        source_id: &str,
        quote: Option<&str>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCoreError> {
        let row = sqlx::query("INSERT INTO task_evidence (task_id, source_type, source_id, quote, confidence) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, task_id, source_type, source_id, quote, confidence::float8 AS confidence, created_at")
            .bind(task_id).bind(source_type).bind(source_id).bind(quote).bind(confidence.unwrap_or(1.0)).fetch_one(&self.pool).await?;
        Ok(TaskEvidence {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            source_type: row.try_get("source_type")?,
            source_id: row.try_get("source_id")?,
            quote: row.try_get("quote")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── TaskRelation ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRelation {
    pub id: String,
    pub task_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskRelationStore {
    pool: PgPool,
}
impl TaskRelationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskRelation>, TaskCoreError> {
        let rows = sqlx::query("SELECT id::text, task_id, entity_type, entity_id, relation_type, source, confidence::float8 AS confidence, created_at FROM task_relations WHERE task_id=$1 ORDER BY relation_type")
            .bind(task_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskRelation {
                    id: r.try_get("id")?,
                    task_id: r.try_get("task_id")?,
                    entity_type: r.try_get("entity_type")?,
                    entity_id: r.try_get("entity_id")?,
                    relation_type: r.try_get("relation_type")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn link(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<TaskRelation, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO task_relations (task_id, entity_type, entity_id, relation_type) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING RETURNING id::text, task_id, entity_type, entity_id, relation_type, source, confidence::float8 AS confidence, created_at")
            .bind(task_id).bind(entity_type).bind(entity_id).bind(relation_type).fetch_one(&mut *transaction).await?;
        let relation = TaskRelation {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        };

        Self::materialize_relationship_in_transaction(&mut transaction, &relation).await?;
        transaction.commit().await?;

        Ok(relation)
    }

    async fn materialize_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relation: &TaskRelation,
    ) -> Result<(), TaskCoreError> {
        let Some(target_entity_kind) = task_relation_entity_kind(&relation.entity_type) else {
            return Ok(());
        };
        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Task,
            source_entity_id: relation.task_id.clone(),
            target_entity_kind,
            target_entity_id: relation.entity_id.clone(),
            relationship_type: relation.relation_type.clone(),
            trust_score: 0.5,
            strength_score: 0.6,
            confidence: relation.confidence,
            review_state: RelationshipReviewState::UserConfirmed,
            valid_from: None,
            valid_to: None,
            metadata: json!({
                "compatibility_table": "task_relations",
                "compatibility_record_id": relation.id,
                "source": relation.source,
                "entity_type": relation.entity_type
            }),
        };
        let evidence = NewRelationshipEvidence::new(
            RelationshipEvidenceSourceKind::RawRecord,
            relation.id.clone(),
        )
        .excerpt("Task relation was recorded through compatibility task relation data.")
        .metadata(json!({
            "compatibility_table": "task_relations",
            "task_id": relation.task_id,
            "entity_type": relation.entity_type,
            "entity_id": relation.entity_id,
            "relation_type": relation.relation_type,
            "source": relation.source
        }));

        RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }
}

// ── TaskChecklist ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskChecklist {
    pub id: String,
    pub task_id: String,
    pub items: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskChecklistStore {
    pool: PgPool,
}
impl TaskChecklistStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get(&self, task_id: &str) -> Result<Option<TaskChecklist>, TaskCoreError> {
        let row = sqlx::query("SELECT id::text, task_id, items, source, created_at, updated_at FROM task_checklists WHERE task_id=$1 ORDER BY created_at DESC LIMIT 1")
            .bind(task_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(TaskChecklist {
                id: r.try_get("id")?,
                task_id: r.try_get("task_id")?,
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
        task_id: &str,
        items: Value,
        source: &str,
    ) -> Result<TaskChecklist, TaskCoreError> {
        let row = sqlx::query("INSERT INTO task_checklists (task_id, items, source) VALUES ($1,$2,$3) RETURNING id::text, task_id, items, source, created_at, updated_at")
            .bind(task_id).bind(&items).bind(source).fetch_one(&self.pool).await?;
        Ok(TaskChecklist {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── TaskSubtasks ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSubtask {
    pub id: String,
    pub parent_task_id: String,
    pub child_task_id: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskSubtaskStore {
    pool: PgPool,
}
impl TaskSubtaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, parent_id: &str) -> Result<Vec<TaskSubtask>, TaskCoreError> {
        let rows = sqlx::query("SELECT id::text, parent_task_id, child_task_id, sort_order, created_at FROM task_subtasks WHERE parent_task_id=$1 ORDER BY sort_order")
            .bind(parent_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskSubtask {
                    id: r.try_get("id")?,
                    parent_task_id: r.try_get("parent_task_id")?,
                    child_task_id: r.try_get("child_task_id")?,
                    sort_order: r.try_get("sort_order")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        parent_id: &str,
        child_id: &str,
        order: i32,
    ) -> Result<TaskSubtask, TaskCoreError> {
        let row = sqlx::query("INSERT INTO task_subtasks (parent_task_id, child_task_id, sort_order) VALUES ($1,$2,$3) ON CONFLICT (parent_task_id, child_task_id) DO UPDATE SET sort_order=$3 RETURNING id::text, parent_task_id, child_task_id, sort_order, created_at")
            .bind(parent_id).bind(child_id).bind(order).fetch_one(&self.pool).await?;
        Ok(TaskSubtask {
            id: row.try_get("id")?,
            parent_task_id: row.try_get("parent_task_id")?,
            child_task_id: row.try_get("child_task_id")?,
            sort_order: row.try_get("sort_order")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum TaskCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error("not found")]
    NotFound,
}

fn task_relation_entity_kind(entity_type: &str) -> Option<RelationshipEntityKind> {
    match entity_type.trim() {
        "person" | "persona" | "contact" => Some(RelationshipEntityKind::Persona),
        "organization" | "org" => Some(RelationshipEntityKind::Organization),
        "project" => Some(RelationshipEntityKind::Project),
        "communication" | "communication_message" | "message" | "email" => {
            Some(RelationshipEntityKind::Communication)
        }
        "document" | "doc" => Some(RelationshipEntityKind::Document),
        "task" => Some(RelationshipEntityKind::Task),
        "event" | "calendar_event" => Some(RelationshipEntityKind::Event),
        "decision" => Some(RelationshipEntityKind::Decision),
        "obligation" => Some(RelationshipEntityKind::Obligation),
        "knowledge" | "knowledge_item" => Some(RelationshipEntityKind::Knowledge),
        _ => None,
    }
}
