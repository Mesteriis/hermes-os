use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonaMemoryError;
use crate::domains::personas::core::link_persona_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaMemoryCard {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct PersonaMemoryCardStore {
    pool: PgPool,
}

impl PersonaMemoryCardStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        persona_id: &str,
    ) -> Result<Vec<PersonaMemoryCard>, PersonaMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, title, description, source, confidence::float8 AS confidence, importance,
             created_at, last_verified_at FROM persona_memory_cards
             WHERE persona_id = $1 ORDER BY importance DESC, created_at DESC",
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_memory_card).collect()
    }

    pub async fn upsert(
        &self,
        persona_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
    ) -> Result<PersonaMemoryCard, PersonaMemoryError> {
        let row = sqlx::query(
            "INSERT INTO persona_memory_cards (persona_id, title, description, source, importance)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, persona_id, title, description, source, confidence::float8 AS confidence, importance,
                       created_at, last_verified_at",
        )
        .bind(persona_id)
        .bind(title)
        .bind(description)
        .bind(source)
        .bind(importance)
        .fetch_one(&self.pool)
        .await?;
        row_to_memory_card(row)
    }

    pub async fn upsert_with_observation(
        &self,
        persona_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
        observation_id: &str,
    ) -> Result<PersonaMemoryCard, PersonaMemoryError> {
        let card = self
            .upsert(persona_id, title, description, source, importance)
            .await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "memory_card",
            card.id.clone(),
            None,
            Some(json!({
                "persona_id": persona_id,
                "importance": card.importance,
            })),
        )
        .await?;
        Ok(card)
    }
}

fn row_to_memory_card(row: PgRow) -> Result<PersonaMemoryCard, PersonaMemoryError> {
    Ok(PersonaMemoryCard {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        importance: row.try_get("importance")?,
        created_at: row.try_get("created_at")?,
        last_verified_at: row.try_get("last_verified_at")?,
    })
}
