mod failures;
mod finish;
mod progress;
mod runs;
mod settings;
mod status;

pub use progress::MailSyncTrigger;
pub use runs::{MailSyncFailureReason, MailSyncRun, MailSyncRunResponse};
pub use settings::{MailSyncDueAccount, MailSyncSettings, MailSyncSettingsUpdate};
pub use status::MailSyncStatus;

pub(in crate::domains::mail::background_sync) use failures::SanitizedSyncFailure;
pub(in crate::domains::mail::background_sync) use finish::FinishRun;
pub(in crate::domains::mail::background_sync) use progress::{
    MailSyncPhase, MailSyncRunStatus, ProgressMode, ProgressUpdate,
};
