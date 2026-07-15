use chrono::{DateTime, Utc};

use super::super::errors::MailSyncError;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::rows::row_to_run;
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn recover_interrupted_runs(&self, now: DateTime<Utc>) -> Result<u64, MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'skipped',
                phase = 'skipped',
                progress_mode = 'none',
                progress_percent = NULL,
                error_code = NULL,
                error_message = NULL,
                completed_at = $1,
                next_run_at = $1,
                updated_at = $1
            WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed')
               OR (status = 'failed' AND error_code = 'backend_restarted')
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
                upserted_personas,
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
        .bind(now)
        .fetch_all(&mut *transaction)
        .await?;
        let affected = rows.len() as u64;
        for row in rows {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "interrupted_requeued",
                now,
                "mail.background_sync.recover_interrupted_runs",
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(affected)
    }
}
