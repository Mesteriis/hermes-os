use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::errors::MailSyncError;
use super::models::{
    FinishRun, MailSyncDueAccount, MailSyncRun, MailSyncRunResponse, MailSyncSettings,
    MailSyncSettingsUpdate, MailSyncStatus, MailSyncTrigger, ProgressUpdate,
};
use super::rows::{row_to_due_account, row_to_run, row_to_settings, row_to_status};
use super::validation::{mail_sync_run_id, validate_account_id, validate_settings};
use super::{DEFAULT_MAIL_SYNC_BATCH_SIZE, DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS};

#[derive(Clone)]
pub struct MailSyncStore {
    pool: PgPool,
}

impl MailSyncStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mark_orphaned_active_runs_failed(
        &self,
        now: DateTime<Utc>,
    ) -> Result<u64, MailSyncError> {
        let result = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'failed',
                phase = 'failed',
                progress_mode = 'none',
                progress_percent = NULL,
                error_code = 'backend_restarted',
                error_message = 'Mail sync run was interrupted by backend restart',
                completed_at = $1,
                next_run_at = $1,
                updated_at = $1
            WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed')
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn settings_for_account(
        &self,
        account_id: &str,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (account_id, batch_size, poll_interval_seconds)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id) DO UPDATE SET account_id = EXCLUDED.account_id
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
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
        validate_settings(update.batch_size, update.poll_interval_seconds)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (
                account_id,
                sync_enabled,
                batch_size,
                poll_interval_seconds,
                updated_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                sync_enabled = EXCLUDED.sync_enabled,
                batch_size = EXCLUDED.batch_size,
                poll_interval_seconds = EXCLUDED.poll_interval_seconds,
                updated_at = now()
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(update.sync_enabled)
        .bind(update.batch_size)
        .bind(update.poll_interval_seconds)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }

    pub async fn sync_statuses(&self) -> Result<Vec<MailSyncStatus>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    phase,
                    progress_mode,
                    progress_percent,
                    processed_messages,
                    estimated_total_messages,
                    current_batch_size,
                    started_at,
                    completed_at,
                    next_run_at,
                    error_code,
                    error_message,
                    fetched_messages,
                    projected_messages,
                    upserted_persons,
                    upserted_organizations
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(latest.status, 'idle') AS status,
                COALESCE(latest.phase, 'idle') AS phase,
                COALESCE(latest.progress_mode, 'none') AS progress_mode,
                latest.progress_percent,
                COALESCE(latest.processed_messages, 0) AS processed_messages,
                latest.estimated_total_messages,
                COALESCE(latest.current_batch_size, COALESCE(settings.batch_size, $1)) AS current_batch_size,
                latest.started_at AS last_started_at,
                latest.completed_at AS last_completed_at,
                COALESCE(
                    latest.next_run_at,
                    CASE
                        WHEN COALESCE(settings.sync_enabled, true) THEN now()
                        ELSE NULL
                    END
                ) AS next_run_at,
                latest.error_code AS last_error_code,
                latest.error_message AS last_error_message,
                COALESCE(latest.fetched_messages, 0) AS last_fetched_messages,
                COALESCE(latest.projected_messages, 0) AS last_projected_messages,
                COALESCE(latest.upserted_persons, 0) AS last_upserted_persons,
                COALESCE(latest.upserted_organizations, 0) AS last_upserted_organizations
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
            ORDER BY a.display_name ASC, a.account_id ASC
            "#,
        )
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_status).collect()
    }

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

    pub(super) async fn start_run(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
    ) -> Result<MailSyncRun, MailSyncError> {
        validate_account_id(account_id)?;
        let run_id = mail_sync_run_id(account_id);
        let result = sqlx::query(
            r#"
            INSERT INTO communication_mail_sync_runs (
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                current_batch_size,
                checkpoint_before
            )
            VALUES ($1, $2, $3, 'running', 'listing', 'indeterminate', $4, $5)
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(&run_id)
        .bind(account_id.trim())
        .bind(trigger.as_str())
        .bind(settings.batch_size)
        .bind(checkpoint_before.unwrap_or_else(|| json!({})))
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => row_to_run(row),
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(MailSyncError::RunAlreadyActive)
            }
            Err(error) => Err(MailSyncError::Sqlx(error)),
        }
    }

    pub(super) async fn update_progress(
        &self,
        update: ProgressUpdate<'_>,
    ) -> Result<(), MailSyncError> {
        sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'running',
                phase = $2,
                progress_mode = $3,
                progress_percent = $4,
                processed_messages = $5,
                estimated_total_messages = $6,
                current_batch_size = $7,
                updated_at = now()
            WHERE run_id = $1
            "#,
        )
        .bind(update.run_id)
        .bind(update.phase.as_str())
        .bind(update.progress_mode.as_str())
        .bind(update.progress_percent)
        .bind(update.processed_messages)
        .bind(update.estimated_total_messages)
        .bind(update.current_batch_size)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub(super) async fn mark_recoverable_full_resync(
        &self,
        run_id: &str,
        error_code: &'static str,
    ) -> Result<(), MailSyncError> {
        sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'recoverable_full_resync_needed',
                phase = 'listing',
                progress_mode = 'indeterminate',
                error_code = $2,
                error_message = 'Gmail history expired; restarting full mailbox listing',
                updated_at = now()
            WHERE run_id = $1
            "#,
        )
        .bind(run_id)
        .bind(error_code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub(super) async fn finish_run(
        &self,
        run_id: &str,
        finish: FinishRun,
    ) -> Result<MailSyncRun, MailSyncError> {
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = $2,
                phase = $3,
                progress_mode = $4,
                progress_percent = $5,
                processed_messages = $6,
                estimated_total_messages = $7,
                fetched_messages = $8,
                projected_messages = $9,
                upserted_persons = $10,
                upserted_organizations = $11,
                checkpoint_after = $12,
                checkpoint_saved = $13,
                error_code = $14,
                error_message = $15,
                completed_at = now(),
                next_run_at = $16,
                updated_at = now()
            WHERE run_id = $1
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(run_id)
        .bind(finish.status.as_str())
        .bind(finish.phase.as_str())
        .bind(finish.progress_mode.as_str())
        .bind(finish.progress_percent)
        .bind(finish.processed_messages)
        .bind(finish.estimated_total_messages)
        .bind(finish.fetched_messages)
        .bind(finish.projected_messages)
        .bind(finish.upserted_persons)
        .bind(finish.upserted_organizations)
        .bind(finish.checkpoint_after.unwrap_or_else(|| json!({})))
        .bind(finish.checkpoint_saved)
        .bind(finish.error_code)
        .bind(finish.error_message)
        .bind(finish.next_run_at)
        .fetch_one(&self.pool)
        .await?;

        row_to_run(row)
    }

    pub(super) async fn latest_run_response(
        &self,
        account_id: &str,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let row = sqlx::query(
            r#"
            SELECT
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            FROM communication_mail_sync_runs
            WHERE account_id = $1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MailSyncError::RunNotFound);
        };

        row_to_run(row).map(Into::into)
    }

    async fn require_account(&self, account_id: &str) -> Result<(), MailSyncError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM communication_provider_accounts WHERE account_id = $1)",
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;
        if exists {
            Ok(())
        } else {
            Err(MailSyncError::AccountNotFound)
        }
    }
}
