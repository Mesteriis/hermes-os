use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::engines::memory::MemoryEngine;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonFact {
    pub id: String,
    pub person_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonFactStore {
    pool: PgPool,
}

impl PersonFactStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonFact>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at,
             valid_from, valid_to, is_active, created_at, updated_at
             FROM person_facts WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_fact).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<PersonFact, PersonMemoryError> {
        let fact =
            MemoryEngine::persona_fact_memory(person_id, fact_type, value, source, confidence)?;
        let row = sqlx::query(
            "INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, fact_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, valid_from, valid_to, is_active, created_at, updated_at",
        )
        .bind(&fact.affected_entity_id)
        .bind(&fact.fact_type)
        .bind(&fact.value)
        .bind(&fact.source)
        .bind(fact.confidence)
        .fetch_one(&self.pool)
        .await?;
        row_to_fact(row)
    }

    pub async fn update_confidence(
        &self,
        id: &str,
        confidence: f64,
    ) -> Result<(), PersonMemoryError> {
        sqlx::query("UPDATE person_facts SET confidence = $2, last_verified_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).bind(confidence).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, PersonMemoryError> {
        let result = sqlx::query(
            "UPDATE person_facts SET confidence = confidence * 0.5, updated_at = now()
             WHERE last_verified_at IS NULL
                OR last_verified_at < now() - ($1 || ' days')::interval",
        )
        .bind(threshold_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

fn row_to_fact(row: PgRow) -> Result<PersonFact, PersonMemoryError> {
    Ok(PersonFact {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        fact_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
