use chrono::{DateTime, Utc};

use super::super::errors::MailSyncError;
use super::super::models::MailSyncDueAccount;
use super::super::rows::row_to_due_account;
use super::super::{DEFAULT_MAIL_SYNC_BATCH_SIZE, DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS};
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn due_accounts(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<MailSyncDueAccount>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    completed_at,
                    next_run_at
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(settings.batch_size, $2) AS batch_size,
                COALESCE(settings.poll_interval_seconds, $3) AS poll_interval_seconds
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
              AND COALESCE(a.config->>'auth_state', '') <> 'deleted'
              AND NOT (a.config ? 'deleted_at')
              AND COALESCE(settings.sync_enabled, true)
              AND NOT EXISTS (
                  SELECT 1
                  FROM communication_mail_sync_runs active
                  WHERE active.account_id = a.account_id
                    AND active.status IN ('queued', 'running', 'recoverable_full_resync_needed')
              )
              AND (
                  COALESCE(
                      latest.next_run_at,
                      latest.completed_at + (COALESCE(settings.poll_interval_seconds, $3)::text || ' seconds')::interval,
                      $1
                  ) <= $1
              )
            ORDER BY latest.completed_at ASC NULLS FIRST, a.account_id ASC
            LIMIT $4
            "#,
        )
        .bind(now)
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .bind(DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_due_account).collect()
    }
}
