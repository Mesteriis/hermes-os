use chrono::{DateTime, Utc};
use serde::Deserialize;

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
