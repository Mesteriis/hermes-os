use super::super::errors::MailSyncError;
use super::super::models::MailSyncRunResponse;
use super::super::rows::row_to_run;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn latest_run_response(
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
}
