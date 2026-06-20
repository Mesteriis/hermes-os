use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

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

fn checkpoint_is_present(checkpoint: Option<&Value>) -> bool {
    checkpoint
        .and_then(Value::as_object)
        .is_some_and(|object| !object.is_empty())
}
