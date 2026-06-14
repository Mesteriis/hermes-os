use serde_json::json;

use super::constants::EVENT_TARGET_KIND;
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn event_append(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self::new(
            actor_id,
            "event.append",
            "POST",
            "/api/v1/events",
            EVENT_TARGET_KIND,
            Some(event_id.into()),
            json!({}),
        )
    }

    pub fn event_get(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self::new(
            actor_id,
            "event.get",
            "GET",
            "/api/v1/events/{event_id}",
            EVENT_TARGET_KIND,
            Some(event_id.into()),
            json!({}),
        )
    }
}
