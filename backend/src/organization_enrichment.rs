use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgEnrichmentResult {
    pub id: String,
    pub organization_id: String,
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
pub struct OrgEnrichmentStore {
    pool: PgPool,
}
impl OrgEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgEnrichmentResult>, OrgEnrichmentError> {
        let rows = sqlx::query("SELECT id::text, organization_id, source, url, data, confidence, status, last_checked_at, applied_at, created_at FROM organization_enrichment_results WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgEnrichmentResult {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    source: r.try_get("source")?,
                    url: r.try_get("url")?,
                    data: r.try_get("data")?,
                    confidence: r.try_get("confidence")?,
                    status: r.try_get("status")?,
                    last_checked_at: r.try_get("last_checked_at")?,
                    applied_at: r.try_get("applied_at")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        source: &str,
        data: Value,
        confidence: f64,
    ) -> Result<OrgEnrichmentResult, OrgEnrichmentError> {
        let row = sqlx::query("INSERT INTO organization_enrichment_results (organization_id, source, data, confidence) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, source, url, data, confidence, status, last_checked_at, applied_at, created_at")
            .bind(org_id).bind(source).bind(&data).bind(confidence).fetch_one(&self.pool).await?;
        Ok(OrgEnrichmentResult {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            source: row.try_get("source")?,
            url: row.try_get("url")?,
            data: row.try_get("data")?,
            confidence: row.try_get("confidence")?,
            status: row.try_get("status")?,
            last_checked_at: row.try_get("last_checked_at")?,
            applied_at: row.try_get("applied_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
    pub async fn apply(&self, id: &str) -> Result<(), OrgEnrichmentError> {
        sqlx::query("UPDATE organization_enrichment_results SET status='applied', applied_at=now() WHERE id::text=$1").bind(id).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn reject(&self, id: &str) -> Result<(), OrgEnrichmentError> {
        sqlx::query(
            "UPDATE organization_enrichment_results SET status='rejected' WHERE id::text=$1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum OrgEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
