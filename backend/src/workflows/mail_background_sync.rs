pub const DEFAULT_MAIL_SYNC_BATCH_SIZE: i32 = 100;
pub const DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS: i32 = 300;

const MAX_BATCH_SIZE: i32 = 500;
const MIN_POLL_INTERVAL_SECONDS: i32 = 60;
const MAX_POLL_INTERVAL_SECONDS: i32 = 86_400;
pub const DEFAULT_GMAIL_API_BASE_URL: &str = "https://www.googleapis.com";

mod errors;
mod events;
mod evidence;
mod idle;
mod models;
mod provider;
mod rows;
mod service;
mod store;
mod validation;

pub use self::errors::MailSyncError;
pub use self::idle::MailImapIdleOutcome;
pub use self::models::{
    MailSyncDueAccount, MailSyncFailureReason, MailSyncRun, MailSyncRunResponse, MailSyncSettings,
    MailSyncSettingsUpdate, MailSyncStatus, MailSyncTrigger,
};
pub use self::service::MailBackgroundSyncService;
pub use self::store::MailSyncStore;
