#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SensitiveForwardingSuppression {
    Disabled,
    Expired,
    BelowMinimumSeverity,
    QuietHours,
    RateLimited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSensitiveForwardingPolicy {
    pub policy_id: String,
    pub source_account_id: String,
    pub delivery_account_id: String,
    pub name: String,
    pub enabled: bool,
    pub include_message_body: bool,
    pub include_attachments: bool,
    pub fixed_recipients: Vec<String>,
    pub minimum_severity: String,
    pub subject_template: String,
    pub body_template: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct StoredSensitiveForwardingPolicy {
    pub policy_id: String,
    pub source_account_id: String,
    pub delivery_account_id: String,
    pub name: String,
    pub enabled: bool,
    pub include_message_body: bool,
    pub include_attachments: bool,
    pub fixed_recipients: Vec<String>,
    pub minimum_severity: String,
    pub subject_template: String,
    pub body_template: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveForwardingRequest {
    pub dispatch_id: String,
    pub policy_id: String,
    pub source_account_id: String,
    pub message_id: String,
    pub severity: String,
    pub has_unsafe_attachments: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SensitiveForwardingDispatchReport {
    pub queued: usize,
    pub already_dispatched: usize,
    pub suppressed: usize,
}
use serde_json::Value;
