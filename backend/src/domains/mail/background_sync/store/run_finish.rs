use serde_json::json;

use super::super::errors::MailSyncError;
use super::super::models::{FinishRun, MailSyncRun};
use super::super::rows::row_to_run;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::domains::mail::background_sync) async fn finish_run(
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
}
