use serde_json::Value;

use crate::workflows::email_sync_pipeline::EmailSyncPipelineReport;

#[derive(Default)]
pub(in crate::workflows::mail_background_sync) struct ProviderSyncSummary {
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) current_batch_size: i32,
    pub(in crate::workflows::mail_background_sync) fetched_messages: i64,
    pub(in crate::workflows::mail_background_sync) projected_messages: i64,
    pub(in crate::workflows::mail_background_sync) upserted_persons: i64,
    pub(in crate::workflows::mail_background_sync) upserted_organizations: i64,
    pub(in crate::workflows::mail_background_sync) checkpoint_after: Option<Value>,
    pub(in crate::workflows::mail_background_sync) checkpoint_saved: bool,
}

impl ProviderSyncSummary {
    pub(in crate::workflows::mail_background_sync::provider) fn apply_pipeline_report(
        &mut self,
        report: &EmailSyncPipelineReport,
    ) {
        self.projected_messages += report.projected_messages as i64;
        self.upserted_persons += report.upserted_person_identities as i64;
        self.upserted_organizations += report.upserted_organizations as i64;
    }
}
