use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub email: Option<String>,
    pub credentials_reference: Option<String>,
    pub sync_status: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CalendarAccountUpdate {
    pub account_name: Option<String>,
    pub email: Option<String>,
    pub sync_status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarSource {
    pub source_id: String,
    pub account_id: String,
    pub provider_calendar_id: Option<String>,
    pub name: String,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub visibility: String,
    pub read_only: bool,
    pub sync_enabled: bool,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
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
pub struct NewCalendarEvent {
    pub source_event_id: Option<String>,
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub all_day: Option<bool>,
    pub recurrence_rule: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub event_type: Option<String>,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CalendarEventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub all_day: Option<bool>,
    pub recurrence_rule: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub event_type: Option<String>,
    pub importance_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
}
