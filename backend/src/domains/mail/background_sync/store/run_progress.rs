use super::super::errors::MailSyncError;
use super::super::models::ProgressUpdate;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::domains::mail::background_sync) async fn update_progress(
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

    pub(in crate::domains::mail::background_sync) async fn mark_recoverable_full_resync(
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
}
