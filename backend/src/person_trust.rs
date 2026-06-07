use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

// ── PersonPromises ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPromise {
    pub id: String,
    pub person_id: String,
    pub description: String,
    pub source_message_id: Option<String>,
    pub promised_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPromiseStore {
    pool: PgPool,
}

impl PersonPromiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPromise>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, description, source_message_id, promised_at,
             due_at, fulfilled_at, status, created_at, updated_at
             FROM person_promises WHERE person_id = $1 ORDER BY promised_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_promise).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        description: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<PersonPromise, PersonTrustError> {
        let row = sqlx::query(
            "INSERT INTO person_promises (person_id, description, due_at)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, description, source_message_id, promised_at,
                       due_at, fulfilled_at, status, created_at, updated_at",
        )
        .bind(person_id)
        .bind(description)
        .bind(due_at)
        .fetch_one(&self.pool)
        .await?;
        row_to_promise(row)
    }

    pub async fn fulfill(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query("UPDATE person_promises SET status = 'fulfilled', fulfilled_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn mark_broken(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises SET status = 'broken', updated_at = now() WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_promise(row: PgRow) -> Result<PersonPromise, PersonTrustError> {
    Ok(PersonPromise {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        description: row.try_get("description")?,
        source_message_id: row.try_get("source_message_id")?,
        promised_at: row.try_get("promised_at")?,
        due_at: row.try_get("due_at")?,
        fulfilled_at: row.try_get("fulfilled_at")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// ── PersonRisks ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRisk {
    pub id: String,
    pub person_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Clone)]
pub struct PersonRiskStore {
    pool: PgPool,
}

impl PersonRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonRisk>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, risk_type, description, severity, source, confidence,
             created_at, resolved_at, resolution
             FROM person_risks WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_risk).collect()
    }

    pub async fn report(
        &self,
        person_id: &str,
        risk_type: &str,
        description: &str,
        severity: &str,
        source: &str,
    ) -> Result<PersonRisk, PersonTrustError> {
        let row = sqlx::query(
            "INSERT INTO person_risks (person_id, risk_type, description, severity, source)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id::text, person_id, risk_type, description, severity, source, confidence,
                       created_at, resolved_at, resolution",
        )
        .bind(person_id)
        .bind(risk_type)
        .bind(description)
        .bind(severity)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_risk(row)
    }

    pub async fn resolve(&self, id: &str, resolution: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_risks SET resolved_at = now(), resolution = $2 WHERE id::text = $1",
        )
        .bind(id)
        .bind(resolution)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_risk(row: PgRow) -> Result<PersonRisk, PersonTrustError> {
    Ok(PersonRisk {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        risk_type: row.try_get("risk_type")?,
        description: row.try_get("description")?,
        severity: row.try_get("severity")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        created_at: row.try_get("created_at")?,
        resolved_at: row.try_get("resolved_at")?,
        resolution: row.try_get("resolution")?,
    })
}

#[derive(Debug, Error)]
pub enum PersonTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
