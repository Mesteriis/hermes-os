use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonCoreError;
use super::preferences::{
    delete_interaction_preferences_in_transaction,
    materialize_interaction_preferences_in_transaction,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPersonaStore {
    pool: PgPool,
}

impl PersonPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonPersona>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas WHERE person_id = $1 ORDER BY name"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn upsert(
        &self,
        persona: &NewPersonPersona,
    ) -> Result<PersonPersona, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO person_personas (persona_id, person_id, name, context, default_tone,
               default_language, preferred_channel)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (persona_id)
               DO UPDATE SET name = EXCLUDED.name, context = EXCLUDED.context,
                             default_tone = EXCLUDED.default_tone,
                             default_language = EXCLUDED.default_language,
                             preferred_channel = EXCLUDED.preferred_channel,
                             updated_at = now()
               RETURNING persona_id, person_id, name, context, default_tone, default_language,
                         preferred_channel, metadata, created_at, updated_at"#,
        )
        .bind(&persona.persona_id)
        .bind(&persona.person_id)
        .bind(&persona.name)
        .bind(&persona.context)
        .bind(&persona.default_tone)
        .bind(&persona.default_language)
        .bind(&persona.preferred_channel)
        .fetch_one(&mut *transaction)
        .await?;
        let persona = row_to_persona(row)?;

        materialize_interaction_preferences_in_transaction(&mut transaction, &persona).await?;
        transaction.commit().await?;

        Ok(persona)
    }

    pub async fn delete(&self, persona_id: &str) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let existing_persona = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas
               WHERE persona_id = $1
               FOR UPDATE"#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .map(row_to_persona)
        .transpose()?;

        let result = sqlx::query("DELETE FROM person_personas WHERE persona_id = $1")
            .bind(persona_id)
            .execute(&mut *transaction)
            .await?;
        let deleted = result.rows_affected() > 0;

        if let Some(existing_persona) = existing_persona
            && deleted
        {
            delete_interaction_preferences_in_transaction(&mut transaction, &existing_persona)
                .await?;
        }

        transaction.commit().await?;

        Ok(deleted)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewPersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
}

fn row_to_persona(row: PgRow) -> Result<PersonPersona, PersonCoreError> {
    Ok(PersonPersona {
        persona_id: row.try_get("persona_id")?,
        person_id: row.try_get("person_id")?,
        name: row.try_get("name")?,
        context: row.try_get("context")?,
        default_tone: row.try_get("default_tone")?,
        default_language: row.try_get("default_language")?,
        preferred_channel: row.try_get("preferred_channel")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
