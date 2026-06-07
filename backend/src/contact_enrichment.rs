use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::contact_intelligence::CommunicationFingerprint;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedContact {
    pub contact_id: String,
    pub display_name: String,
    pub email_address: String,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub trust_score: Option<i16>,
    pub avg_response_hours: Option<f64>,
    pub preferred_channel: Option<String>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub frequent_topics: Vec<String>,
    pub writing_style: Option<String>,
    pub contact_metadata: Value,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub linked_projects: Vec<String>,
    pub linked_documents: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ContactEnrichmentStore {
    pool: PgPool,
}

impl ContactEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enrich_contact(
        &self,
        contact_id: &str,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<EnrichedContact, ContactEnrichmentError> {
        let row = sqlx::query(
            r#"UPDATE contacts SET
                language = COALESCE($2, contacts.language),
                tone = COALESCE($3, contacts.tone),
                trust_score = COALESCE($4, contacts.trust_score),
                avg_response_hours = COALESCE($5, contacts.avg_response_hours),
                writing_style = COALESCE($6, contacts.writing_style),
                updated_at = now()
            WHERE contact_id = $1
            RETURNING contact_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, contact_metadata, is_favorite, notes, created_at, updated_at"#,
        ).bind(contact_id).bind(fingerprint.detected_language.as_deref()).bind(fingerprint.typical_tone.as_deref()).bind(fingerprint.trust_score).bind(fingerprint.avg_response_hours).bind(fingerprint.writing_style.as_deref()).fetch_optional(&self.pool).await?;

        let Some(row) = row else {
            return Err(ContactEnrichmentError::NotFound);
        };
        row_to_enriched(row)
    }

    pub async fn get_enriched(
        &self,
        contact_id: &str,
    ) -> Result<Option<EnrichedContact>, ContactEnrichmentError> {
        let row = sqlx::query(
            r#"SELECT c.contact_id, c.display_name, c.email_address, c.language, c.tone, c.trust_score, c.avg_response_hours, c.preferred_channel, c.last_interaction_at, c.interaction_count, c.frequent_topics, c.writing_style, c.contact_metadata, c.is_favorite, c.notes, c.created_at, c.updated_at
            FROM contacts c WHERE c.contact_id = $1"#,
        ).bind(contact_id).fetch_optional(&self.pool).await?;
        row.map(row_to_enriched).transpose()
    }

    pub async fn list_enriched(
        &self,
        favorites_only: bool,
        limit: i64,
    ) -> Result<Vec<EnrichedContact>, ContactEnrichmentError> {
        let limit = limit.clamp(1, 100);
        let rows = if favorites_only {
            sqlx::query("SELECT contact_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, contact_metadata, is_favorite, notes, created_at, updated_at FROM contacts WHERE is_favorite = true ORDER BY trust_score DESC NULLS LAST, interaction_count DESC LIMIT $1").bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT contact_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, contact_metadata, is_favorite, notes, created_at, updated_at FROM contacts ORDER BY interaction_count DESC, trust_score DESC NULLS LAST LIMIT $1").bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(row_to_enriched).collect()
    }

    pub async fn toggle_favorite(&self, contact_id: &str) -> Result<bool, ContactEnrichmentError> {
        let row = sqlx::query("UPDATE contacts SET is_favorite = NOT is_favorite, updated_at = now() WHERE contact_id = $1 RETURNING is_favorite").bind(contact_id).fetch_optional(&self.pool).await?;
        Ok(row
            .map(|r: PgRow| r.try_get("is_favorite").unwrap_or(false))
            .unwrap_or(false))
    }

    pub async fn set_notes(
        &self,
        contact_id: &str,
        notes: &str,
    ) -> Result<(), ContactEnrichmentError> {
        sqlx::query("UPDATE contacts SET notes = $2, updated_at = now() WHERE contact_id = $1")
            .bind(contact_id)
            .bind(notes)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_contacts(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<EnrichedContact>, ContactEnrichmentError> {
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let rows = sqlx::query("SELECT contact_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, contact_metadata, is_favorite, notes, created_at, updated_at FROM contacts WHERE lower(display_name) LIKE $1 OR lower(email_address) LIKE $1 ORDER BY interaction_count DESC LIMIT $2").bind(&pattern).bind(limit.clamp(1, 100)).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enriched).collect()
    }
}

fn row_to_enriched(row: PgRow) -> Result<EnrichedContact, ContactEnrichmentError> {
    Ok(EnrichedContact {
        contact_id: row.try_get("contact_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        language: row.try_get("language")?,
        tone: row.try_get("tone")?,
        trust_score: row.try_get("trust_score")?,
        avg_response_hours: row.try_get("avg_response_hours")?,
        preferred_channel: row.try_get("preferred_channel")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
        interaction_count: row.try_get("interaction_count")?,
        frequent_topics: serde_json::from_value(row.try_get("frequent_topics")?)
            .unwrap_or_default(),
        writing_style: row.try_get("writing_style")?,
        contact_metadata: row.try_get("contact_metadata")?,
        is_favorite: row.try_get("is_favorite")?,
        notes: row.try_get("notes")?,
        linked_projects: vec![],
        linked_documents: vec![],
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum ContactEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("contact not found")]
    NotFound,
}
