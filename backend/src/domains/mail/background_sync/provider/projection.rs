use crate::domains::graph::projection::GraphProjectionService;
use crate::domains::mail::storage::LocalMailBlobStore;
use crate::domains::mail::sync::EmailSyncBatch;
use crate::workflows::email_sync_pipeline::project_email_sync_batch_with_mail_blobs;

use super::super::errors::ProviderSyncError;
use super::super::models::{MailSyncPhase, MailSyncSettings, ProgressMode, ProgressUpdate};
use super::super::service::MailBackgroundSyncService;
use super::super::store::MailSyncStore;
use super::ProviderSyncSummary;

impl MailBackgroundSyncService {
    pub(in crate::domains::mail::background_sync::provider) async fn project_batch(
        &self,
        store: &MailSyncStore,
        run_id: &str,
        settings: &MailSyncSettings,
        summary: &mut ProviderSyncSummary,
        account_id: &str,
        batch: EmailSyncBatch,
    ) -> Result<(), ProviderSyncError> {
        let fetched_count = batch.messages.len() as i64;
        summary.fetched_messages += fetched_count;
        summary.processed_messages += fetched_count;
        summary.current_batch_size = i32::try_from(fetched_count).unwrap_or(i32::MAX);
        summary.checkpoint_after = batch.checkpoint.clone();

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::Projecting,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        let blob_store = LocalMailBlobStore::new(&self.blob_root);
        let report = project_email_sync_batch_with_mail_blobs(
            self.pool.clone(),
            &blob_store,
            account_id,
            &format!("{run_id}:batch:{}", summary.processed_messages),
            &batch,
        )
        .await?;
        summary.apply_pipeline_report(&report);

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::PersonsGraph,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        GraphProjectionService::new(self.pool.clone())
            .project_from_v1()
            .await?;

        if batch.checkpoint.is_some() {
            summary.checkpoint_saved = report.checkpoint_saved;
        }

        Ok(())
    }
}
