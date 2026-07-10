use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonaCoreError;
use super::link_persona_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaIdentity {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: Option<String>,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonaIdentityStore {
    pool: PgPool,
}

impl PersonaIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        persona_id: &str,
    ) -> Result<Vec<PersonaIdentity>, PersonaCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, persona_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM persona_identities WHERE persona_id = $1 ORDER BY identity_type"#,
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn find_active_persona_id(
        &self,
        identity_type: &str,
        identity_value: &str,
    ) -> Result<Option<String>, PersonaCoreError> {
        let row = sqlx::query(
            r#"
            SELECT persona_id
            FROM persona_identities
            WHERE identity_type = $1
              AND identity_value = $2
              AND status = 'active'
              AND persona_id IS NOT NULL
            ORDER BY updated_at DESC, id
            LIMIT 1
            "#,
        )
        .bind(identity_type)
        .bind(identity_value)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| row.try_get("persona_id")).transpose()?)
    }

    pub async fn list_unattached(
        &self,
        limit: i64,
    ) -> Result<Vec<PersonaIdentity>, PersonaCoreError> {
        let limit = limit.clamp(1, 200);
        let rows = sqlx::query(
            r#"SELECT id::text, persona_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM persona_identities
               WHERE persona_id IS NULL
               ORDER BY updated_at DESC, id
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn upsert(
        &self,
        persona_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO persona_identities (persona_id, identity_type, identity_value, source)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, persona_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(persona_id)
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn upsert_with_observation(
        &self,
        persona_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let identity = self
            .upsert(persona_id, identity_type, identity_value, source)
            .await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "identity",
            identity.id.clone(),
            None,
            Some(json!({
                "persona_id": identity.persona_id,
                "identity_type": identity.identity_type,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn create_unattached(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        self.create_unattached_with_metadata(identity_type, identity_value, source, json!({}))
            .await
    }

    pub async fn create_unattached_with_metadata(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        metadata: Value,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO persona_identities (
                   persona_id, identity_type, identity_value, source, metadata
               )
               VALUES (NULL, $1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET
                   metadata = persona_identities.metadata || EXCLUDED.metadata,
                   updated_at = now()
               RETURNING id::text, persona_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn create_unattached_with_observation(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let identity = self
            .create_unattached(identity_type, identity_value, source)
            .await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            None,
            Some(json!({
                "identity_type": identity.identity_type,
                "persona_id": identity.persona_id,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn create_unattached_with_metadata_and_observation(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        metadata: Value,
        observation_id: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let identity = self
            .create_unattached_with_metadata(identity_type, identity_value, source, metadata)
            .await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            None,
            Some(json!({
                "identity_type": identity.identity_type,
                "persona_id": identity.persona_id,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn attach_to_persona(
        &self,
        identity_id: &str,
        persona_id: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let row = sqlx::query(
            r#"UPDATE persona_identities
               SET persona_id = $2, status = 'active', updated_at = now()
               WHERE id::text = $1
               RETURNING id::text, persona_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_id)
        .bind(persona_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersonaCoreError::IdentityNotFound)?;
        row_to_identity(row)
    }

    pub async fn attach_to_persona_with_observation(
        &self,
        identity_id: &str,
        persona_id: &str,
        observation_id: &str,
    ) -> Result<PersonaIdentity, PersonaCoreError> {
        let identity = self.attach_to_persona(identity_id, persona_id).await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            Some("trace_assignment"),
            Some(json!({
                "persona_id": identity.persona_id,
                "identity_type": identity.identity_type,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn update_status(
        &self,
        identity_id: &str,
        status: &str,
    ) -> Result<(), PersonaCoreError> {
        sqlx::query(
            "UPDATE persona_identities SET status = $2, updated_at = now() WHERE id::text = $1",
        )
        .bind(identity_id)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, identity_id: &str) -> Result<bool, PersonaCoreError> {
        let result = sqlx::query("DELETE FROM persona_identities WHERE id::text = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_with_observation(
        &self,
        persona_id: &str,
        identity_id: &str,
        observation_id: &str,
    ) -> Result<bool, PersonaCoreError> {
        let deleted = self.delete(identity_id).await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "identity",
            identity_id.to_owned(),
            Some("identity_delete"),
            Some(json!({
                "persona_id": persona_id,
                "deleted": deleted,
            })),
        )
        .await?;
        Ok(deleted)
    }
}

fn row_to_identity(row: PgRow) -> Result<PersonaIdentity, PersonaCoreError> {
    Ok(PersonaIdentity {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
        identity_type: row.try_get("identity_type")?,
        identity_value: row.try_get("identity_value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        status: row.try_get("status")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
