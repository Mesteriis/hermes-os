use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonMemoryCard {
    pub id: String,
    pub person_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct PersonMemoryCardStore {
    pool: PgPool,
}

impl PersonMemoryCardStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonMemoryCard>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
             created_at, last_verified_at FROM person_memory_cards
             WHERE person_id = $1 ORDER BY importance DESC, created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_memory_card).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
    ) -> Result<PersonMemoryCard, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_memory_cards (person_id, title, description, source, importance)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
                       created_at, last_verified_at",
        )
        .bind(person_id)
        .bind(title)
        .bind(description)
        .bind(source)
        .bind(importance)
        .fetch_one(&self.pool)
        .await?;
        row_to_memory_card(row)
    }
}

fn row_to_memory_card(row: PgRow) -> Result<PersonMemoryCard, PersonMemoryError> {
    Ok(PersonMemoryCard {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        importance: row.try_get("importance")?,
        created_at: row.try_get("created_at")?,
        last_verified_at: row.try_get("last_verified_at")?,
    })
}
