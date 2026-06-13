use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domains::mail::sync::EmailSyncPlanError;
use crate::vault::HostVaultError;

use super::errors::ProviderSyncError;
use super::validation::next_run_at;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncSettings {
    pub account_id: String,
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct MailSyncSettingsUpdate {
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncStatus {
    pub account_id: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub last_fetched_messages: i64,
    pub last_projected_messages: i64,
    pub last_upserted_persons: i64,
    pub last_upserted_organizations: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncRun {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before: Option<Value>,
    pub checkpoint_after: Option<Value>,
    pub checkpoint_saved: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncRunResponse {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before_present: bool,
    pub checkpoint_after_present: bool,
    pub checkpoint_saved: bool,
    pub failure_reason: Option<MailSyncFailureReason>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

impl From<MailSyncRun> for MailSyncRunResponse {
    fn from(run: MailSyncRun) -> Self {
        Self {
            run_id: run.run_id,
            account_id: run.account_id,
            trigger: run.trigger,
            status: run.status,
            phase: run.phase,
            progress_mode: run.progress_mode,
            progress_percent: run.progress_percent,
            processed_messages: run.processed_messages,
            estimated_total_messages: run.estimated_total_messages,
            current_batch_size: run.current_batch_size,
            fetched_messages: run.fetched_messages,
            projected_messages: run.projected_messages,
            upserted_persons: run.upserted_persons,
            upserted_organizations: run.upserted_organizations,
            checkpoint_before_present: checkpoint_is_present(run.checkpoint_before.as_ref()),
            checkpoint_after_present: checkpoint_is_present(run.checkpoint_after.as_ref()),
            checkpoint_saved: run.checkpoint_saved,
            failure_reason: run.error_code.map(|code| MailSyncFailureReason {
                code,
                message: run
                    .error_message
                    .unwrap_or_else(|| "Mail sync failed".to_owned()),
            }),
            started_at: run.started_at,
            completed_at: run.completed_at,
            next_run_at: run.next_run_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncFailureReason {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncDueAccount {
    pub account_id: String,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncTrigger {
    Scheduled,
    Manual,
}

impl MailSyncTrigger {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

pub(super) struct ProgressUpdate<'a> {
    pub(super) run_id: &'a str,
    pub(super) phase: MailSyncPhase,
    pub(super) progress_mode: ProgressMode,
    pub(super) progress_percent: Option<i32>,
    pub(super) processed_messages: i64,
    pub(super) estimated_total_messages: Option<i64>,
    pub(super) current_batch_size: i32,
}

#[derive(Clone, Copy)]
pub(super) enum MailSyncRunStatus {
    Completed,
    Failed,
    Skipped,
}

impl MailSyncRunStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum MailSyncPhase {
    Listing,
    Fetching,
    Projecting,
    PersonsGraph,
    Completed,
    Failed,
}

impl MailSyncPhase {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::Fetching => "fetching",
            Self::Projecting => "projecting",
            Self::PersonsGraph => "persons_graph",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ProgressMode {
    None,
    Determinate,
    Indeterminate,
}

impl ProgressMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Determinate => "determinate",
            Self::Indeterminate => "indeterminate",
        }
    }
}

pub(super) struct FinishRun {
    pub(super) status: MailSyncRunStatus,
    pub(super) phase: MailSyncPhase,
    pub(super) progress_mode: ProgressMode,
    pub(super) progress_percent: Option<i32>,
    pub(super) processed_messages: i64,
    pub(super) estimated_total_messages: Option<i64>,
    pub(super) fetched_messages: i64,
    pub(super) projected_messages: i64,
    pub(super) upserted_persons: i64,
    pub(super) upserted_organizations: i64,
    pub(super) checkpoint_after: Option<Value>,
    pub(super) checkpoint_saved: bool,
    pub(super) error_code: Option<String>,
    pub(super) error_message: Option<String>,
    pub(super) next_run_at: Option<DateTime<Utc>>,
}

impl FinishRun {
    pub(super) fn failed(
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

#[derive(Debug)]
pub(super) struct SanitizedSyncFailure {
    code: String,
    message: String,
}

impl SanitizedSyncFailure {
    pub(super) fn from_plan(error: EmailSyncPlanError) -> Self {
        tracing::warn!(error = %error, "mail sync provider configuration is invalid");
        Self {
            code: "provider_config_invalid".to_owned(),
            message: "Mail provider configuration is invalid".to_owned(),
        }
    }

    pub(super) fn from_vault(error: HostVaultError) -> Self {
        match error {
            HostVaultError::Locked => Self {
                code: "vault_locked".to_owned(),
                message: "Host vault is locked".to_owned(),
            },
            HostVaultError::Uninitialized => Self {
                code: "vault_uninitialized".to_owned(),
                message: "Host vault is not initialized".to_owned(),
            },
            other => {
                tracing::warn!(error = %other, "mail sync vault check failed");
                Self {
                    code: "vault_unavailable".to_owned(),
                    message: "Host vault is unavailable".to_owned(),
                }
            }
        }
    }
}

impl From<ProviderSyncError> for SanitizedSyncFailure {
    fn from(error: ProviderSyncError) -> Self {
        match error {
            ProviderSyncError::MissingCredential | ProviderSyncError::Credential(_) => Self {
                code: "credential_unavailable".to_owned(),
                message: "Provider credential is unavailable for this account".to_owned(),
            },
            ProviderSyncError::AccountSetup(_) => Self {
                code: "oauth_refresh_failed".to_owned(),
                message: "OAuth access token refresh failed".to_owned(),
            },
            ProviderSyncError::ProviderNetwork(error) => {
                tracing::warn!(error = %error, "mail provider sync network call failed");
                Self {
                    code: "provider_network_error".to_owned(),
                    message: "Mail provider network request failed".to_owned(),
                }
            }
            ProviderSyncError::Pipeline(error) => {
                tracing::error!(error = %error, "mail sync projection pipeline failed");
                Self {
                    code: "projection_failed".to_owned(),
                    message: "Mail sync projection failed".to_owned(),
                }
            }
            ProviderSyncError::Graph(error) => {
                tracing::error!(error = %error, "mail sync graph projection failed");
                Self {
                    code: "graph_projection_failed".to_owned(),
                    message: "Mail graph projection failed".to_owned(),
                }
            }
            ProviderSyncError::Communication(error) => {
                tracing::error!(error = %error, "mail sync communication store failed");
                Self {
                    code: "communication_store_error".to_owned(),
                    message: "Mail sync communication store failed".to_owned(),
                }
            }
            ProviderSyncError::SyncStore(error) => {
                tracing::error!(error = %error, "mail sync status store failed");
                Self {
                    code: "sync_store_error".to_owned(),
                    message: "Mail sync status store failed".to_owned(),
                }
            }
        }
    }
}

fn checkpoint_is_present(checkpoint: Option<&Value>) -> bool {
    checkpoint
        .and_then(Value::as_object)
        .is_some_and(|object| !object.is_empty())
}
