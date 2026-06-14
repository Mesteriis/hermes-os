use sqlx::Row;

use super::errors::CommunicationIngestionError;
use super::models::{
    DeletedProviderAccount, NewProviderAccount, ProviderAccount, ProviderAccountUsage,
};
use super::rows::row_to_provider_account;
use super::store::CommunicationIngestionStore;
use super::validation::{validate_non_empty, validate_object};
use serde_json::Value;

impl CommunicationIngestionStore {
    pub async fn upsert_provider_account(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        account.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = now()
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account.account_id.trim())
        .bind(account.provider_kind.as_str())
        .bind(account.display_name.trim())
        .bind(account.external_account_id.trim())
        .bind(&account.config)
        .fetch_one(&self.pool)
        .await?;

        row_to_provider_account(row)
    }

    pub async fn provider_account(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_account).transpose()
    }

    pub async fn list_provider_accounts(
        &self,
    ) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            ORDER BY provider_kind ASC, display_name ASC, account_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_account).collect()
    }

    pub async fn update_provider_account_config(
        &self,
        account_id: &str,
        config: &Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_object("config", config)?;

        let row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(config)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_account).transpose()
    }

    pub async fn provider_account_usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                (SELECT count(*) FROM communication_raw_records WHERE account_id = $1) AS raw_record_count,
                (SELECT count(*) FROM communication_messages WHERE account_id = $1) AS message_count,
                (SELECT count(*) FROM communication_ingestion_checkpoints WHERE account_id = $1) AS checkpoint_count
            "#,
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(ProviderAccountUsage {
            raw_record_count: row.try_get("raw_record_count")?,
            message_count: row.try_get("message_count")?,
            checkpoint_count: row.try_get("checkpoint_count")?,
        })
    }

    pub async fn delete_provider_account_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let mut transaction = self.pool.begin().await?;

        let binding_rows = sqlx::query(
            r#"
            DELETE FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            RETURNING secret_ref
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&mut *transaction)
        .await?;
        let unbound_secret_refs = binding_rows
            .into_iter()
            .map(|row| row.try_get("secret_ref"))
            .collect::<Result<Vec<String>, sqlx::Error>>()?;

        sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .execute(&mut *transaction)
        .await?;

        let account_row = sqlx::query(
            r#"
            DELETE FROM communication_provider_accounts
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(DeletedProviderAccount {
            account: account_row.map(row_to_provider_account).transpose()?,
            unbound_secret_refs,
        })
    }
}
