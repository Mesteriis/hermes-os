use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CanonicalMessageVersionRecord {
    pub version_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub version_number: i32,
    pub body_text: Option<String>,
    pub edit_timestamp: DateTime<Utc>,
    pub source_event: Option<String>,
    pub raw_diff_payload: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalMessageTombstoneRecord {
    pub tombstone_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub reason_class: String,
    pub actor_class: String,
    pub observed_at: DateTime<Utc>,
    pub source_event: Option<String>,
    pub is_provider_delete: bool,
    pub is_local_visible: bool,
    pub metadata: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalMessageReactionRecord {
    pub reaction_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub reaction_emoji: String,
    pub is_active: bool,
    pub observed_at: DateTime<Utc>,
    pub source_event: Option<String>,
    pub provider_actor_id: Option<String>,
    pub metadata: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalMessageReferenceSummaryRecord {
    pub message_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub chat_title: Option<String>,
    pub sender: Option<String>,
    pub sender_display_name: Option<String>,
    pub text: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalReplyReferenceRecord {
    pub reply_ref_id: String,
    pub source_message_id: String,
    pub target_message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
    pub reply_depth: i32,
    pub is_topic_reply: bool,
    pub topic_id: Option<String>,
    pub metadata: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalForwardReferenceRecord {
    pub forward_ref_id: String,
    pub source_message_id: String,
    pub target_message_id: Option<String>,
    pub account_id: String,
    pub provider_chat_id: String,
    pub source_provider_id: String,
    pub forward_origin_chat_id: Option<String>,
    pub forward_origin_message_id: Option<String>,
    pub forward_origin_sender_id: Option<String>,
    pub forward_origin_sender_name: Option<String>,
    pub forward_date: Option<DateTime<Utc>>,
    pub forward_depth: i32,
    pub metadata: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum CanonicalReadPortError {
    #[error("canonical communication read failed: {0}")]
    Storage(String),
}

#[async_trait::async_trait]
pub trait CanonicalMessageReadPort: Send + Sync {
    async fn list_message_versions(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageVersionRecord>, CanonicalReadPortError>;
    async fn list_message_tombstones(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageTombstoneRecord>, CanonicalReadPortError>;
    async fn list_message_reactions(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageReactionRecord>, CanonicalReadPortError>;
    async fn list_message_reference_summaries(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<CanonicalMessageReferenceSummaryRecord>, CanonicalReadPortError>;
    async fn list_reply_references_by_target(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalReplyReferenceRecord>, CanonicalReadPortError>;
    async fn list_reply_references_by_source(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalReplyReferenceRecord>, CanonicalReadPortError>;
    async fn list_forward_references_by_source(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalForwardReferenceRecord>, CanonicalReadPortError>;
}
