use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::PersonaCoreError;
use super::evidence::link_persona_entity_in_transaction;
use super::preferences::{
    delete_interaction_preferences_in_transaction,
    materialize_interaction_preferences_in_transaction,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaInteractionContext {
    #[serde(alias = "persona_id")]
    pub interaction_context_id: String,
    #[serde(alias = "person_id")]
    pub source_persona_id: String,
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
pub struct PersonaInteractionContextStore {
    pool: PgPool,
}

impl PersonaInteractionContextStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        persona_id: &str,
    ) -> Result<Vec<PersonaInteractionContext>, PersonaCoreError> {
        let rows = sqlx::query(
            r#"SELECT interaction_context_id, source_persona_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM persona_interaction_contexts WHERE source_persona_id = $1 ORDER BY name"#,
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn upsert(
        &self,
        persona: &NewPersonaInteractionContext,
    ) -> Result<PersonaInteractionContext, PersonaCoreError> {
        self.upsert_with_source(persona, None).await
    }

    pub async fn upsert_with_source(
        &self,
        persona: &NewPersonaInteractionContext,
        source: Option<&str>,
    ) -> Result<PersonaInteractionContext, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let persona = Self::upsert_in_transaction(&mut transaction, persona, source).await?;
        transaction.commit().await?;

        Ok(persona)
    }

    pub async fn upsert_with_observation(
        &self,
        persona: &NewPersonaInteractionContext,
        source: Option<&str>,
        observation_id: &str,
    ) -> Result<PersonaInteractionContext, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let persona = Self::upsert_in_transaction(&mut transaction, persona, source).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona.interaction_context_id.clone(),
            None,
            Some(json!({
                "interaction_context_id": persona.interaction_context_id,
                "source_persona_id": persona.source_persona_id,
                "action": "upsert",
            })),
        )
        .await?;
        transaction.commit().await?;

        Ok(persona)
    }

    async fn upsert_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona: &NewPersonaInteractionContext,
        source: Option<&str>,
    ) -> Result<PersonaInteractionContext, PersonaCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO persona_interaction_contexts (interaction_context_id, source_persona_id, name, context, default_tone,
               default_language, preferred_channel)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (interaction_context_id)
               DO UPDATE SET name = EXCLUDED.name, context = EXCLUDED.context,
                             default_tone = EXCLUDED.default_tone,
                             default_language = EXCLUDED.default_language,
                             preferred_channel = EXCLUDED.preferred_channel,
                             updated_at = now()
               RETURNING interaction_context_id, source_persona_id, name, context, default_tone, default_language,
                         preferred_channel, metadata, created_at, updated_at"#,
        )
        .bind(&persona.interaction_context_id)
        .bind(&persona.source_persona_id)
        .bind(&persona.name)
        .bind(&persona.context)
        .bind(&persona.default_tone)
        .bind(&persona.default_language)
        .bind(&persona.preferred_channel)
        .fetch_one(&mut **transaction)
        .await?;
        let persona = row_to_persona(row)?;

        let source = source
            .map(str::to_owned)
            .unwrap_or_else(|| interaction_context_source(&persona.interaction_context_id));
        materialize_interaction_preferences_in_transaction(transaction, &persona, &source).await?;

        Ok(persona)
    }

    pub async fn delete(&self, persona_id: &str) -> Result<bool, PersonaCoreError> {
        self.delete_with_source(persona_id, None).await
    }

    pub async fn delete_with_source(
        &self,
        persona_id: &str,
        source: Option<&str>,
    ) -> Result<bool, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let deleted = Self::delete_in_transaction(&mut transaction, persona_id, source).await?;
        transaction.commit().await?;
        Ok(deleted)
    }

    pub async fn delete_with_observation(
        &self,
        source_persona_id: &str,
        interaction_context_id: &str,
        source: Option<&str>,
        observation_id: &str,
    ) -> Result<bool, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let deleted =
            Self::delete_in_transaction(&mut transaction, interaction_context_id, source).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            interaction_context_id.to_owned(),
            None,
            Some(json!({
                "interaction_context_id": interaction_context_id,
                "source_persona_id": source_persona_id,
                "action": "delete",
                "deleted": deleted,
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(deleted)
    }

    async fn delete_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona_id: &str,
        source: Option<&str>,
    ) -> Result<bool, PersonaCoreError> {
        let existing_persona = sqlx::query(
            r#"SELECT interaction_context_id, source_persona_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM persona_interaction_contexts
               WHERE interaction_context_id = $1
               FOR UPDATE"#,
        )
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?
        .map(row_to_persona)
        .transpose()?;

        let result = sqlx::query(
            "DELETE FROM persona_interaction_contexts WHERE interaction_context_id = $1",
        )
        .bind(persona_id)
        .execute(&mut **transaction)
        .await?;
        let deleted = result.rows_affected() > 0;

        if let Some(existing_persona) = existing_persona
            && deleted
        {
            let source = source.map(str::to_owned).unwrap_or_else(|| {
                interaction_context_source(&existing_persona.interaction_context_id)
            });
            delete_interaction_preferences_in_transaction(transaction, &existing_persona, &source)
                .await?;
        }

        Ok(deleted)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewPersonaInteractionContext {
    #[serde(alias = "persona_id")]
    pub interaction_context_id: String,
    #[serde(alias = "person_id")]
    pub source_persona_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
}

fn row_to_persona(row: PgRow) -> Result<PersonaInteractionContext, PersonaCoreError> {
    Ok(PersonaInteractionContext {
        interaction_context_id: row.try_get("interaction_context_id")?,
        source_persona_id: row.try_get("source_persona_id")?,
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

fn interaction_context_source(persona_id: &str) -> String {
    format!("persona_interaction_contexts:{persona_id}")
}
