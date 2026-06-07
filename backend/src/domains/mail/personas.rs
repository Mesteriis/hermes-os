use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailPersona {
    pub persona_id: String,
    pub name: String,
    pub account_id: String,
    pub display_name: String,
    pub signature: String,
    pub default_language: Option<String>,
    pub default_tone: Option<String>,
    pub is_default: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EmailPersonaStore {
    pool: PgPool,
}

impl EmailPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        persona: &NewEmailPersona,
    ) -> Result<EmailPersona, EmailPersonaError> {
        persona.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO email_personas (persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (persona_id) DO UPDATE SET
                name = EXCLUDED.name, account_id = EXCLUDED.account_id, display_name = EXCLUDED.display_name,
                signature = EXCLUDED.signature, default_language = EXCLUDED.default_language,
                default_tone = EXCLUDED.default_tone, is_default = EXCLUDED.is_default,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at"#,
        )
        .bind(&persona.persona_id).bind(&persona.name).bind(&persona.account_id)
        .bind(&persona.display_name).bind(&persona.signature)
        .bind(persona.default_language.as_deref()).bind(persona.default_tone.as_deref())
        .bind(persona.is_default).bind(&persona.metadata)
        .fetch_one(&self.pool).await?;
        row_to_persona(row)
    }

    pub async fn list(&self) -> Result<Vec<EmailPersona>, EmailPersonaError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM email_personas ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn get_default(&self) -> Result<Option<EmailPersona>, EmailPersonaError> {
        let row = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM email_personas WHERE is_default = true LIMIT 1"#,
        ).fetch_optional(&self.pool).await?;
        row.map(row_to_persona).transpose()
    }
}

fn row_to_persona(row: PgRow) -> Result<EmailPersona, EmailPersonaError> {
    Ok(EmailPersona {
        persona_id: row.try_get("persona_id")?,
        name: row.try_get("name")?,
        account_id: row.try_get("account_id")?,
        display_name: row.try_get("display_name")?,
        signature: row.try_get("signature")?,
        default_language: row.try_get("default_language")?,
        default_tone: row.try_get("default_tone")?,
        is_default: row.try_get("is_default")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewEmailPersona {
    pub persona_id: String,
    pub name: String,
    pub account_id: String,
    pub display_name: String,
    pub signature: String,
    pub default_language: Option<String>,
    pub default_tone: Option<String>,
    pub is_default: bool,
    pub metadata: Value,
}

impl NewEmailPersona {
    fn validate(&self) -> Result<(), EmailPersonaError> {
        if self.persona_id.trim().is_empty() {
            return Err(EmailPersonaError::Invalid("persona_id empty"));
        }
        if self.name.trim().is_empty() {
            return Err(EmailPersonaError::Invalid("name empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(EmailPersonaError::Invalid("account_id empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum EmailPersonaError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid persona: {0}")]
    Invalid(&'static str),
}
