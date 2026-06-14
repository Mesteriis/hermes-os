use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn application_setting_set(
        actor_id: impl Into<String>,
        setting_key: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "application_setting.set",
            "PUT",
            "/api/v1/settings/{setting_key}",
            "application_setting",
            Some(setting_key.into()),
            json!({}),
        )
    }
}
