use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationPersona {
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
pub struct CommunicationPersonaStore {
    pool: PgPool,
}

impl CommunicationPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        persona: &NewCommunicationPersona,
    ) -> Result<CommunicationPersona, CommunicationPersonaError> {
        persona.validate()?;
        ensure_canonical_account(&self.pool, &persona.account_id).await?;
        let row = sqlx::query(
            r#"INSERT INTO communication_personas (persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata)
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

    pub async fn list(&self) -> Result<Vec<CommunicationPersona>, CommunicationPersonaError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM communication_personas ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn get_default(
        &self,
    ) -> Result<Option<CommunicationPersona>, CommunicationPersonaError> {
        let row = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM communication_personas WHERE is_default = true LIMIT 1"#,
        ).fetch_optional(&self.pool).await?;
        row.map(row_to_persona).transpose()
    }
}

async fn ensure_canonical_account(
    pool: &PgPool,
    account_id: &str,
) -> Result<(), CommunicationPersonaError> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn row_to_persona(row: PgRow) -> Result<CommunicationPersona, CommunicationPersonaError> {
    Ok(CommunicationPersona {
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
pub struct NewCommunicationPersona {
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

impl NewCommunicationPersona {
    fn validate(&self) -> Result<(), CommunicationPersonaError> {
        if self.persona_id.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("persona_id empty"));
        }
        if self.name.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("name empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("account_id empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CommunicationPersonaError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid persona: {0}")]
    Invalid(&'static str),
}
