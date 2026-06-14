use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonIdentity {
    pub id: String,
    pub person_id: Option<String>,
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
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities WHERE person_id = $1 ORDER BY identity_type"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn list_unattached(
        &self,
        limit: i64,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let limit = limit.clamp(1, 200);
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities
               WHERE person_id IS NULL
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
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(person_id)
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn create_unattached(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (person_id, identity_type, identity_value, source)
               VALUES (NULL, $1, $2, $3)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn attach_to_persona(
        &self,
        identity_id: &str,
        person_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"UPDATE person_identities
               SET person_id = $2, status = 'active', updated_at = now()
               WHERE id::text = $1
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_id)
        .bind(person_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersonCoreError::IdentityNotFound)?;
        row_to_identity(row)
    }

    pub async fn update_status(
        &self,
        identity_id: &str,
        status: &str,
    ) -> Result<(), PersonCoreError> {
        sqlx::query(
            "UPDATE person_identities SET status = $2, updated_at = now() WHERE id::text = $1",
        )
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
