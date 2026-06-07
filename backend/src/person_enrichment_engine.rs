use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub id: String,
    pub person_id: String,
    pub source: String,
    pub url: Option<String>,
    pub data: Value,
    pub confidence: f64,
    pub status: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EnrichmentResultStore {
    pool: PgPool,
}

impl EnrichmentResultStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn list(&self, person_id: &str) -> Result<Vec<EnrichmentResult>, EnrichmentEngineError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, source, url, data, confidence, status, last_checked_at, applied_at, created_at
             FROM enrichment_results WHERE person_id = $1 ORDER BY created_at DESC"
        ).bind(person_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enrichment).collect()
    }

    pub async fn upsert(
        &self, person_id: &str, source: &str, data: Value, confidence: f64,
    ) -> Result<EnrichmentResult, EnrichmentEngineError> {
        let row = sqlx::query(
            "INSERT INTO enrichment_results (person_id, source, data, confidence)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, source, url, data, confidence, status, last_checked_at, applied_at, created_at"
        ).bind(person_id).bind(source).bind(&data).bind(confidence).fetch_one(&self.pool).await?;
        row_to_enrichment(row)
    }

    pub async fn apply(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'applied', applied_at = now() WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn reject(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'rejected' WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }
}

fn row_to_enrichment(row: PgRow) -> Result<EnrichmentResult, EnrichmentEngineError> {
    Ok(EnrichmentResult {
        id: row.try_get("id")?, person_id: row.try_get("person_id")?,
        source: row.try_get("source")?, url: row.try_get("url")?,
        data: row.try_get("data")?, confidence: row.try_get("confidence")?,
        status: row.try_get("status")?, last_checked_at: row.try_get("last_checked_at")?,
        applied_at: row.try_get("applied_at")?, created_at: row.try_get("created_at")?,
    })
}

#[derive(Debug, Error)]
pub enum EnrichmentEngineError {
    #[error(transparent)] Sqlx(#[from] sqlx::Error),
    #[error("enrichment not found")] NotFound,
}
