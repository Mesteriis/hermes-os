use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn communication_email_send(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        recipient_count: usize,
    ) -> Self {
        Self::new(
            actor_id,
            "communication.email.send",
            "POST",
            "/api/v1/communications/send",
            "communication_provider_account",
            Some(account_id.into()),
            json!({
                "action_class": "provider_write",
                "transport": "smtp",
                "recipient_count": recipient_count,
            }),
        )
    }
}
