use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::SecretReferenceError;
use super::models::{NewSecretReference, SecretKind, SecretReference, SecretStoreKind};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct SecretReferenceStore {
    pool: PgPool,
}

impl SecretReferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_secret_reference(
        &self,
        reference: &NewSecretReference,
    ) -> Result<SecretReference, SecretReferenceError> {
        reference.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO secret_references (
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (secret_ref)
            DO UPDATE SET
                secret_kind = EXCLUDED.secret_kind,
                store_kind = EXCLUDED.store_kind,
                label = EXCLUDED.label,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(reference.secret_ref.trim())
        .bind(reference.secret_kind.as_str())
        .bind(reference.store_kind.as_str())
        .bind(reference.label.trim())
        .bind(&reference.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_secret_reference(row)
    }

    pub async fn secret_reference(
        &self,
        secret_ref: &str,
    ) -> Result<Option<SecretReference>, SecretReferenceError> {
        validate_non_empty("secret_ref", secret_ref)?;

        let row = sqlx::query(
            r#"
            SELECT
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            FROM secret_references
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_secret_reference).transpose()
    }

    pub async fn delete_secret_reference(
        &self,
        secret_ref: &str,
    ) -> Result<bool, SecretReferenceError> {
        validate_non_empty("secret_ref", secret_ref)?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM secret_references
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref.trim())
        .execute(&self.pool)
        .await?;

        Ok(deleted.rows_affected() > 0)
    }
}

fn row_to_secret_reference(row: PgRow) -> Result<SecretReference, SecretReferenceError> {
    let secret_kind = SecretKind::try_from(row.try_get::<String, _>("secret_kind")?.as_str())?;
    let store_kind = SecretStoreKind::try_from(row.try_get::<String, _>("store_kind")?.as_str())?;

    Ok(SecretReference {
        secret_ref: row.try_get("secret_ref")?,
        secret_kind,
        store_kind,
        label: row.try_get("label")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
