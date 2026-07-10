use chrono::{DateTime, Utc};
use serde::Serialize;

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
    pub last_updated_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub consecutive_failures: i64,
    pub last_fetched_messages: i64,
    pub last_projected_messages: i64,
    pub last_upserted_personas: i64,
    pub last_upserted_organizations: i64,
}
