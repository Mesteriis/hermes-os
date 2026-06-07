use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

// ── EventParticipant ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventParticipant {
    pub id: String,
    pub event_id: String,
    pub person_id: Option<String>,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub response_status: String,
    pub organization_id: Option<String>,
    pub timezone: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventParticipantStore {
    pool: PgPool,
}

impl EventParticipantStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventParticipant>, CalendarCoreError> {
        let rows = sqlx::query("SELECT id::text, event_id, person_id, email, display_name, role, response_status, organization_id, timezone, confidence, created_at FROM event_participants WHERE event_id=$1 ORDER BY role, email")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(EventParticipant {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    person_id: r.try_get("person_id")?,
                    email: r.try_get("email")?,
                    display_name: r.try_get("display_name")?,
                    role: r.try_get("role")?,
                    response_status: r.try_get("response_status")?,
                    organization_id: r.try_get("organization_id")?,
                    timezone: r.try_get("timezone")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        person_id: Option<&str>,
        org_id: Option<&str>,
    ) -> Result<EventParticipant, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_participants (event_id, email, display_name, role, person_id, organization_id) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, event_id, person_id, email, display_name, role, response_status, organization_id, timezone, confidence, created_at")
            .bind(event_id).bind(email).bind(display_name).bind(role.unwrap_or("attendee")).bind(person_id).bind(org_id).fetch_one(&self.pool).await?;
        Ok(EventParticipant {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            person_id: row.try_get("person_id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            role: row.try_get("role")?,
            response_status: row.try_get("response_status")?,
            organization_id: row.try_get("organization_id")?,
            timezone: row.try_get("timezone")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── EventRelation ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRelation {
    pub id: String,
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventRelationStore {
    pool: PgPool,
}

impl EventRelationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventRelation>, CalendarCoreError> {
        let rows = sqlx::query("SELECT id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at FROM event_relations WHERE event_id=$1 ORDER BY entity_type")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(EventRelation {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
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
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<EventRelation, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_relations (event_id, entity_type, entity_id, relation_type) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING RETURNING id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at")
            .bind(event_id).bind(entity_type).bind(entity_id).bind(relation_type).fetch_one(&self.pool).await?;
        Ok(EventRelation {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── EventContextPack ──────────────────────────────────────────────────────

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

// ── EventAgenda ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventAgenda {
    pub id: String,
    pub event_id: String,
    pub items: Value,
    pub source: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventAgendaStore {
    pool: PgPool,
}

impl EventAgendaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventAgenda>, CalendarCoreError> {
        let row = sqlx::query("SELECT id::text, event_id, items, source, created_by, created_at, updated_at FROM event_agendas WHERE event_id=$1 ORDER BY created_at DESC LIMIT 1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(EventAgenda {
                id: r.try_get("id")?,
                event_id: r.try_get("event_id")?,
                items: r.try_get("items")?,
                source: r.try_get("source")?,
                created_by: r.try_get("created_by")?,
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
    ) -> Result<EventAgenda, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_agendas (event_id, items, source) VALUES ($1,$2,$3) RETURNING id::text, event_id, items, source, created_by, created_at, updated_at")
            .bind(event_id).bind(&items).bind(source).fetch_one(&self.pool).await?;
        Ok(EventAgenda {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_by: row.try_get("created_by")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── EventChecklist ─────────────────────────────────────────────────────────

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
        let row = sqlx::query("INSERT INTO event_checklists (event_id, items, source) VALUES ($1,$2,$3) RETURNING id::text, event_id, items, source, created_at, updated_at")
            .bind(event_id).bind(&items).bind(source).fetch_one(&self.pool).await?;
        Ok(EventChecklist {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum CalendarCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
