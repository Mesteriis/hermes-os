use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CalendarEventRead {
    pub event_id: String,
    pub observation_id: String,
    pub source_event_id: Option<String>,
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub all_day: bool,
    pub recurrence_rule: Option<String>,
    pub status: String,
    pub visibility: String,
    pub event_type: Option<String>,
    pub importance_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub sync_status: String,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CalendarEventListQuery {
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
}

pub type CalendarEventListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<CalendarEventRead>, CalendarQueryError>> + Send + 'a>>;
pub trait CalendarEventReadPort: Send + Sync {
    fn list<'a>(&'a self, query: CalendarEventListQuery) -> CalendarEventListFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
#[error("calendar query failed: {0}")]
pub struct CalendarQueryError(pub String);
