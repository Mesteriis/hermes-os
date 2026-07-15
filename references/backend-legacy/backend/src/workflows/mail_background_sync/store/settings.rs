use super::super::errors::MailSyncError;
use super::super::models::settings::{MailSyncSettings, MailSyncSettingsUpdate};
use super::super::rows::row_to_settings;
use super::super::validation::{validate_account_id, validate_settings};
use super::super::{DEFAULT_MAIL_SYNC_BATCH_SIZE, DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS};
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn settings_for_account(
        &self,
        account_id: &str,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (account_id, batch_size, poll_interval_seconds, failure_threshold)
            VALUES ($1, $2, $3, 3)
            ON CONFLICT (account_id) DO UPDATE SET account_id = EXCLUDED.account_id
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, failure_threshold, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .bind(DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }

    pub async fn update_settings(
        &self,
        account_id: &str,
        update: MailSyncSettingsUpdate,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        validate_settings(
            update.batch_size,
            update.poll_interval_seconds,
            update.failure_threshold,
        )?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (
                account_id,
                sync_enabled,
                batch_size,
                poll_interval_seconds,
                failure_threshold,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                sync_enabled = EXCLUDED.sync_enabled,
                batch_size = EXCLUDED.batch_size,
                poll_interval_seconds = EXCLUDED.poll_interval_seconds,
                failure_threshold = EXCLUDED.failure_threshold,
                updated_at = now()
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, failure_threshold, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(update.sync_enabled)
        .bind(update.batch_size)
        .bind(update.poll_interval_seconds)
        .bind(update.failure_threshold)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }
}
