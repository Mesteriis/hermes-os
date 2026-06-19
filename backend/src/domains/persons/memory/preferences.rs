use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::domains::persons::core::link_persons_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPreference {
    pub id: String,
    pub person_id: String,
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPreferenceStore {
    pool: PgPool,
}

impl PersonPreferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPreference>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
             last_verified_at, created_at, updated_at FROM person_preferences
             WHERE person_id = $1 ORDER BY preference_type",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_preference).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        source: &str,
    ) -> Result<PersonPreference, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (person_id, preference_type) DO UPDATE SET value = $3, source = $4, updated_at = now()
             RETURNING id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, created_at, updated_at"
        ).bind(person_id).bind(preference_type).bind(value).bind(source).fetch_one(&self.pool).await?;
        row_to_preference(row)
    }

    pub async fn upsert_with_observation(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonPreference, PersonMemoryError> {
        let pref = self
            .upsert(person_id, preference_type, value, source)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "preference",
            pref.id.clone(),
            None,
            Some(json!({
                "person_id": person_id,
                "preference_type": pref.preference_type,
            })),
        )
        .await?;
        Ok(pref)
    }
}

fn row_to_preference(row: PgRow) -> Result<PersonPreference, PersonMemoryError> {
    Ok(PersonPreference {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        preference_type: row.try_get("preference_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
