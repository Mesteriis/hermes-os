use chrono::{DateTime, Utc};
use serde_json::Value;

use super::failures::SanitizedSyncFailure;
use super::progress::{MailSyncPhase, MailSyncRunStatus, ProgressMode};
use super::settings::MailSyncSettings;
use crate::workflows::mail_background_sync::validation::next_run_at;

pub(in crate::workflows::mail_background_sync) struct FinishRun {
    pub(in crate::workflows::mail_background_sync) status: MailSyncRunStatus,
    pub(in crate::workflows::mail_background_sync) phase: MailSyncPhase,
    pub(in crate::workflows::mail_background_sync) progress_mode: ProgressMode,
    pub(in crate::workflows::mail_background_sync) progress_percent: Option<i32>,
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) fetched_messages: i64,
    pub(in crate::workflows::mail_background_sync) projected_messages: i64,
    pub(in crate::workflows::mail_background_sync) upserted_persons: i64,
    pub(in crate::workflows::mail_background_sync) upserted_organizations: i64,
    pub(in crate::workflows::mail_background_sync) checkpoint_after: Option<Value>,
    pub(in crate::workflows::mail_background_sync) checkpoint_saved: bool,
    pub(in crate::workflows::mail_background_sync) error_code: Option<String>,
    pub(in crate::workflows::mail_background_sync) error_message: Option<String>,
    pub(in crate::workflows::mail_background_sync) next_run_at: Option<DateTime<Utc>>,
}

impl FinishRun {
    pub(in crate::workflows::mail_background_sync) fn failed(
        phase: MailSyncPhase,
        failure: SanitizedSyncFailure,
        settings: &MailSyncSettings,
    ) -> Self {
        Self {
            status: MailSyncRunStatus::Failed,
            phase,
            progress_mode: ProgressMode::None,
            progress_percent: None,
            processed_messages: 0,
            estimated_total_messages: None,
            fetched_messages: 0,
            projected_messages: 0,
            upserted_persons: 0,
            upserted_organizations: 0,
            checkpoint_after: None,
            checkpoint_saved: false,
            error_code: Some(failure.code),
            error_message: Some(failure.message),
            next_run_at: next_run_at(settings),
        }
    }
}
