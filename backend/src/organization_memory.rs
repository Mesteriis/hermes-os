use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

// ── OrgFact ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgFact { pub id: String, pub organization_id: String, pub fact_type: String, pub value: String, pub source: String, pub confidence: f64, pub last_verified_at: Option<DateTime<Utc>>, pub valid_from: Option<DateTime<Utc>>, pub valid_to: Option<DateTime<Utc>>, pub is_active: bool, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc> }

#[derive(Clone)] pub struct OrgFactStore { pool: PgPool }
impl OrgFactStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgFact>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, fact_type, value, source, confidence, last_verified_at, valid_from, valid_to, is_active, created_at, updated_at FROM organization_facts WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| Ok(OrgFact { id: r.try_get("id")?, organization_id: r.try_get("organization_id")?, fact_type: r.try_get("fact_type")?, value: r.try_get("value")?, source: r.try_get("source")?, confidence: r.try_get("confidence")?, last_verified_at: r.try_get("last_verified_at")?, valid_from: r.try_get("valid_from")?, valid_to: r.try_get("valid_to")?, is_active: r.try_get("is_active")?, created_at: r.try_get("created_at")?, updated_at: r.try_get("updated_at")? })).collect()
    }
    pub async fn upsert(&self, org_id: &str, fact_type: &str, value: &str, source: &str, confidence: f64) -> Result<OrgFact, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_facts (organization_id, fact_type, value, source, confidence) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, fact_type, value, source, confidence, last_verified_at, valid_from, valid_to, is_active, created_at, updated_at")
            .bind(org_id).bind(fact_type).bind(value).bind(source).bind(confidence).fetch_one(&self.pool).await?;
        Ok(OrgFact { id: row.try_get("id")?, organization_id: row.try_get("organization_id")?, fact_type: row.try_get("fact_type")?, value: row.try_get("value")?, source: row.try_get("source")?, confidence: row.try_get("confidence")?, last_verified_at: row.try_get("last_verified_at")?, valid_from: row.try_get("valid_from")?, valid_to: row.try_get("valid_to")?, is_active: row.try_get("is_active")?, created_at: row.try_get("created_at")?, updated_at: row.try_get("updated_at")? })
    }
    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, OrgMemoryError> {
        let result = sqlx::query("UPDATE organization_facts SET confidence = confidence * 0.5, updated_at = now() WHERE last_verified_at IS NULL OR last_verified_at < now() - ($1 || ' days')::interval")
            .bind(threshold_days).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

// ── OrgMemoryCard ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgMemoryCard { pub id: String, pub organization_id: String, pub title: String, pub description: String, pub source: String, pub confidence: f64, pub importance: i16, pub created_at: DateTime<Utc>, pub last_verified_at: Option<DateTime<Utc>> }

#[derive(Clone)] pub struct OrgMemoryCardStore { pool: PgPool }
impl OrgMemoryCardStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgMemoryCard>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, title, description, source, confidence, importance, created_at, last_verified_at FROM organization_memory_cards WHERE organization_id=$1 ORDER BY importance DESC, created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| Ok(OrgMemoryCard { id: r.try_get("id")?, organization_id: r.try_get("organization_id")?, title: r.try_get("title")?, description: r.try_get("description")?, source: r.try_get("source")?, confidence: r.try_get("confidence")?, importance: r.try_get("importance")?, created_at: r.try_get("created_at")?, last_verified_at: r.try_get("last_verified_at")? })).collect()
    }
    pub async fn upsert(&self, org_id: &str, title: &str, description: &str, source: &str, importance: i16) -> Result<OrgMemoryCard, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_memory_cards (organization_id, title, description, source, importance) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, title, description, source, confidence, importance, created_at, last_verified_at")
            .bind(org_id).bind(title).bind(description).bind(source).bind(importance).fetch_one(&self.pool).await?;
        Ok(OrgMemoryCard { id: row.try_get("id")?, organization_id: row.try_get("organization_id")?, title: row.try_get("title")?, description: row.try_get("description")?, source: row.try_get("source")?, confidence: row.try_get("confidence")?, importance: row.try_get("importance")?, created_at: row.try_get("created_at")?, last_verified_at: row.try_get("last_verified_at")? })
    }
}

// ── OrgPreference ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPreference { pub id: String, pub organization_id: String, pub preference_type: String, pub value: String, pub source: String, pub confidence: f64, pub last_verified_at: Option<DateTime<Utc>>, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc> }

#[derive(Clone)] pub struct OrgPreferenceStore { pool: PgPool }
impl OrgPreferenceStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPreference>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, preference_type, value, source, confidence, last_verified_at, created_at, updated_at FROM organization_preferences WHERE organization_id=$1 ORDER BY preference_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| Ok(OrgPreference { id: r.try_get("id")?, organization_id: r.try_get("organization_id")?, preference_type: r.try_get("preference_type")?, value: r.try_get("value")?, source: r.try_get("source")?, confidence: r.try_get("confidence")?, last_verified_at: r.try_get("last_verified_at")?, created_at: r.try_get("created_at")?, updated_at: r.try_get("updated_at")? })).collect()
    }
    pub async fn upsert(&self, org_id: &str, ptype: &str, value: &str, source: &str) -> Result<OrgPreference, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_preferences (organization_id, preference_type, value, source) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id, preference_type) DO UPDATE SET value=$3, source=$4, updated_at=now() RETURNING id::text, organization_id, preference_type, value, source, confidence, last_verified_at, created_at, updated_at")
            .bind(org_id).bind(ptype).bind(value).bind(source).fetch_one(&self.pool).await?;
        Ok(OrgPreference { id: row.try_get("id")?, organization_id: row.try_get("organization_id")?, preference_type: row.try_get("preference_type")?, value: row.try_get("value")?, source: row.try_get("source")?, confidence: row.try_get("confidence")?, last_verified_at: row.try_get("last_verified_at")?, created_at: row.try_get("created_at")?, updated_at: row.try_get("updated_at")? })
    }
}

// ── OrgRequiredDocument ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgRequiredDocument { pub id: String, pub organization_id: String, pub document_type: String, pub description: Option<String>, pub source: String, pub confidence: f64, pub created_at: DateTime<Utc> }

#[derive(Clone)] pub struct OrgRequiredDocStore { pool: PgPool }
impl OrgRequiredDocStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgRequiredDocument>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, document_type, description, source, confidence, created_at FROM organization_required_documents WHERE organization_id=$1 ORDER BY document_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| Ok(OrgRequiredDocument { id: r.try_get("id")?, organization_id: r.try_get("organization_id")?, document_type: r.try_get("document_type")?, description: r.try_get("description")?, source: r.try_get("source")?, confidence: r.try_get("confidence")?, created_at: r.try_get("created_at")? })).collect()
    }
}

#[derive(Debug, Error)] pub enum OrgMemoryError { #[error(transparent)] Sqlx(#[from] sqlx::Error), #[error("not found")] NotFound }
