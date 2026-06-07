use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

// ── TimelineEvent ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTimelineEvent {
    pub id: String,
    pub organization_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTimelineStore {
    pool: PgPool,
}
impl OrgTimelineStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(
        &self,
        org_id: &str,
        limit: i64,
    ) -> Result<Vec<OrgTimelineEvent>, OrgWorkflowError> {
        let rows = sqlx::query("SELECT id::text, organization_id, event_type, title, description, occurred_at, source, related_entity_id, related_entity_kind, confidence, metadata, created_at FROM organization_timeline_events WHERE organization_id=$1 ORDER BY occurred_at DESC LIMIT $2")
            .bind(org_id).bind(limit.clamp(1,100)).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgTimelineEvent {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    event_type: r.try_get("event_type")?,
                    title: r.try_get("title")?,
                    description: r.try_get("description")?,
                    occurred_at: r.try_get("occurred_at")?,
                    source: r.try_get("source")?,
                    related_entity_id: r.try_get("related_entity_id")?,
                    related_entity_kind: r.try_get("related_entity_kind")?,
                    confidence: r.try_get("confidence")?,
                    metadata: r.try_get("metadata")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        event_type: &str,
        title: &str,
        occurred_at: DateTime<Utc>,
        source: &str,
    ) -> Result<OrgTimelineEvent, OrgWorkflowError> {
        let row = sqlx::query("INSERT INTO organization_timeline_events (organization_id, event_type, title, occurred_at, source) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, organization_id, event_type, title, description, occurred_at, source, related_entity_id, related_entity_kind, confidence, metadata, created_at")
            .bind(org_id).bind(event_type).bind(title).bind(occurred_at).bind(source).fetch_one(&self.pool).await?;
        Ok(OrgTimelineEvent {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            event_type: row.try_get("event_type")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            occurred_at: row.try_get("occurred_at")?,
            source: row.try_get("source")?,
            related_entity_id: row.try_get("related_entity_id")?,
            related_entity_kind: row.try_get("related_entity_kind")?,
            confidence: row.try_get("confidence")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── OrgTemplate ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTemplate {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub template_type: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTemplateStore {
    pool: PgPool,
}
impl OrgTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgTemplate>, OrgWorkflowError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, template_type, subject, body, language, tone, metadata, created_at, updated_at FROM organization_templates WHERE organization_id=$1 ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgTemplate {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    template_type: r.try_get("template_type")?,
                    subject: r.try_get("subject")?,
                    body: r.try_get("body")?,
                    language: r.try_get("language")?,
                    tone: r.try_get("tone")?,
                    metadata: r.try_get("metadata")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

// ── OrgPortal ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPortal {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub url: String,
    pub portal_type: String,
    pub login_hint: Option<String>,
    pub secret_reference: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPortalStore {
    pool: PgPool,
}
impl OrgPortalStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPortal>, OrgWorkflowError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, url, portal_type, login_hint, secret_reference, last_used_at, notes, created_at FROM organization_portals WHERE organization_id=$1 ORDER BY portal_type, name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgPortal {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    url: r.try_get("url")?,
                    portal_type: r.try_get("portal_type")?,
                    login_hint: r.try_get("login_hint")?,
                    secret_reference: r.try_get("secret_reference")?,
                    last_used_at: r.try_get("last_used_at")?,
                    notes: r.try_get("notes")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        url: &str,
        portal_type: &str,
    ) -> Result<OrgPortal, OrgWorkflowError> {
        let row = sqlx::query("INSERT INTO organization_portals (organization_id, name, url, portal_type) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, name, url, portal_type, login_hint, secret_reference, last_used_at, notes, created_at")
            .bind(org_id).bind(name).bind(url).bind(portal_type).fetch_one(&self.pool).await?;
        Ok(OrgPortal {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            url: row.try_get("url")?,
            portal_type: row.try_get("portal_type")?,
            login_hint: row.try_get("login_hint")?,
            secret_reference: row.try_get("secret_reference")?,
            last_used_at: row.try_get("last_used_at")?,
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── OrgProcedure ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgProcedure {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Value,
    pub source: String,
    pub confidence: f64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgProcedureStore {
    pool: PgPool,
}
impl OrgProcedureStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgProcedure>, OrgWorkflowError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, description, steps, source, confidence, last_used_at, created_at, updated_at FROM organization_procedures WHERE organization_id=$1 ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgProcedure {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    description: r.try_get("description")?,
                    steps: r.try_get("steps")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    last_used_at: r.try_get("last_used_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

// ── OrgPlaybook ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPlaybook {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub trigger_condition: Option<String>,
    pub steps: Value,
    pub approval_mode: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPlaybookStore {
    pool: PgPool,
}
impl OrgPlaybookStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPlaybook>, OrgWorkflowError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, trigger_condition, steps, approval_mode, enabled, last_run_at, created_at, updated_at FROM organization_playbooks WHERE organization_id=$1 ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgPlaybook {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    trigger_condition: r.try_get("trigger_condition")?,
                    steps: r.try_get("steps")?,
                    approval_mode: r.try_get("approval_mode")?,
                    enabled: r.try_get("enabled")?,
                    last_run_at: r.try_get("last_run_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum OrgWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
