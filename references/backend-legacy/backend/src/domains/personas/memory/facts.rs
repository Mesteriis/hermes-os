use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonaMemoryError;
use crate::domains::personas::core::evidence::link_persona_entity;
use crate::engines::memory::MemoryEngine;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaFact {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
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
pub struct PersonaFactStore {
    pool: PgPool,
}

impl PersonaFactStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, persona_id: &str) -> Result<Vec<PersonaFact>, PersonaMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at,
             valid_from, valid_to, is_active, created_at, updated_at
             FROM persona_facts WHERE persona_id = $1 ORDER BY created_at DESC",
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_fact).collect()
    }

    pub async fn upsert(
        &self,
        persona_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<PersonaFact, PersonaMemoryError> {
        let fact =
            MemoryEngine::persona_fact_memory(persona_id, fact_type, value, source, confidence)?;
        let row = sqlx::query(
            "INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, persona_id, fact_type, value, source, confidence::float8 AS confidence,
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

    pub async fn upsert_with_observation(
        &self,
        persona_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
        observation_id: &str,
    ) -> Result<PersonaFact, PersonaMemoryError> {
        let fact = self
            .upsert(persona_id, fact_type, value, source, confidence)
            .await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "fact",
            fact.id.clone(),
            None,
            Some(json!({
                "persona_id": persona_id,
                "fact_type": fact.fact_type,
            })),
        )
        .await?;
        Ok(fact)
    }

    pub async fn update_confidence(
        &self,
        id: &str,
        confidence: f64,
    ) -> Result<(), PersonaMemoryError> {
        sqlx::query("UPDATE persona_facts SET confidence = $2, last_verified_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).bind(confidence).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, PersonaMemoryError> {
        let result = sqlx::query(
            "UPDATE persona_facts SET confidence = confidence * 0.5, updated_at = now()
             WHERE last_verified_at IS NULL
                OR last_verified_at < now() - ($1 || ' days')::interval",
        )
        .bind(threshold_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

fn row_to_fact(row: PgRow) -> Result<PersonaFact, PersonaMemoryError> {
    Ok(PersonaFact {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
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
