use chrono::{DateTime, Utc};

use super::super::errors::MailSyncError;
use super::MailSyncStore;

impl MailSyncStore {
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
}
