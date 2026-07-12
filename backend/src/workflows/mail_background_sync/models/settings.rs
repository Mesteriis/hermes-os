use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncSettings {
    pub account_id: String,
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
    pub failure_threshold: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct MailSyncSettingsUpdate {
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
    pub failure_threshold: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncDueAccount {
    pub account_id: String,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}
