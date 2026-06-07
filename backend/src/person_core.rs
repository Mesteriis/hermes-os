use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

// ── PersonIdentity ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonIdentity {
    pub id: String,
    pub person_id: String,
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
pub struct PersonsIdentityStore {
    pool: PgPool,
}

impl PersonsIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence, last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities WHERE person_id = $1 ORDER BY identity_type"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (person_id, identity_type, identity_value, source)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence, last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(person_id)
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn update_status(
        &self,
        identity_id: &str,
        status: &str,
    ) -> Result<(), PersonCoreError> {
        sqlx::query("UPDATE person_identities SET status = $2, updated_at = now() WHERE id::text = $1")
            .bind(identity_id)
            .bind(status)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, identity_id: &str) -> Result<bool, PersonCoreError> {
        let result = sqlx::query("DELETE FROM person_identities WHERE id::text = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

fn row_to_identity(row: PgRow) -> Result<PersonIdentity, PersonCoreError> {
    Ok(PersonIdentity {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
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

// ── PersonRole ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRole {
    pub id: String,
    pub person_id: String,
    pub role: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonRoleStore {
    pool: PgPool,
}

impl PersonRoleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(&self, person_id: &str) -> Result<Vec<PersonRole>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, role, assigned_by, assigned_at
               FROM person_roles WHERE person_id = $1 ORDER BY assigned_at"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    pub async fn assign(
        &self,
        person_id: &str,
        role: &str,
        assigned_by: Option<&str>,
    ) -> Result<PersonRole, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_roles (person_id, role, assigned_by)
               VALUES ($1, $2, $3)
               ON CONFLICT (person_id, role) DO UPDATE SET assigned_by = EXCLUDED.assigned_by
               RETURNING id::text, person_id, role, assigned_by, assigned_at"#,
        )
        .bind(person_id)
        .bind(role)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await?;
        row_to_role(row)
    }

    pub async fn remove(&self, person_id: &str, role: &str) -> Result<bool, PersonCoreError> {
        let result = sqlx::query("DELETE FROM person_roles WHERE person_id = $1 AND role = $2")
            .bind(person_id)
            .bind(role)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

fn row_to_role(row: PgRow) -> Result<PersonRole, PersonCoreError> {
    Ok(PersonRole {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        role: row.try_get("role")?,
        assigned_by: row.try_get("assigned_by")?,
        assigned_at: row.try_get("assigned_at")?,
    })
}

// ── PersonPersona ───────────────────────────────────────────────────────────

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
        .fetch_one(&self.pool)
        .await?;
        row_to_persona(row)
    }

    pub async fn delete(&self, persona_id: &str) -> Result<bool, PersonCoreError> {
        let result = sqlx::query("DELETE FROM person_personas WHERE persona_id = $1")
            .bind(persona_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
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

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("person identity not found")]
    IdentityNotFound,
    #[error("person persona not found")]
    PersonaNotFound,
}
