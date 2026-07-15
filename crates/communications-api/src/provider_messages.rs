use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderChannelMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: String,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageAttachmentAnchor {
    pub message_id: String,
    pub raw_record_id: String,
}

pub struct ProviderAttachmentDownloadStateUpdate<'a> {
    pub message_id: &'a str,
    pub provider_attachment_id: &'a str,
    pub communication_attachment_id: Option<&'a str>,
    pub provider_file_id: i64,
    pub download_state: &'a str,
    pub local_path: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub content_type: &'a str,
    pub filename: Option<&'a str>,
    pub observed_at: DateTime<Utc>,
    pub context: ProviderMessageProjectionObservationContext<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageReferenceSummary {
    pub message_id: String,
    pub provider_record_id: String,
    pub conversation_id: Option<String>,
    pub subject: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHeuristicMember {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy)]
pub struct ProviderMessageProjectionObservationContext<'a> {
    pub channel_kinds: &'a [&'a str],
    pub relationship_kind: &'a str,
    pub actor: &'a str,
}

pub struct ProviderMessageObservationEvent<'a> {
    pub provider: &'a str,
    pub account_id: &'a str,
    pub channel_kind: &'a str,
    pub message_id: &'a str,
    pub external_message_id: &'a str,
    pub event_kind: &'a str,
    pub observed_at: DateTime<Utc>,
    pub external_event_id: Option<&'a str>,
    pub payload: &'a Value,
    pub causation_id: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}
