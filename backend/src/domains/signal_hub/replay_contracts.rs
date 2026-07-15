use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalReplayRequest {
    pub id: String,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub from_position: Option<i64>,
    pub to_position: Option<i64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub target_consumer: Option<String>,
    pub target_projection: Option<String>,
    pub status: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error_redacted: Option<String>,
    pub replayed_count: i32,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalReplayRequestCreate {
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub from_position: Option<i64>,
    pub to_position: Option<i64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub target_consumer: Option<String>,
    pub target_projection: Option<String>,
    pub requested_by: String,
    pub metadata: Value,
}
