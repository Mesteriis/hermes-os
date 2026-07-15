use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CanonicalConversationRecord {
    pub conversation_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub provider_conversation_id: String,
    pub title: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CanonicalConversationMemberRecord {
    pub participant_id: String,
    pub display_name: String,
    pub role: String,
    pub address: Option<String>,
    pub participant_metadata: Value,
    pub provider_identity_id: Option<String>,
    pub identity_kind: Option<String>,
    pub identity_metadata: Option<Value>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub conversation_id: Option<String>,
    pub account_id: Option<String>,
    pub provider_conversation_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CanonicalPresenceRecord {
    pub identity_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub provider_identity_id: Option<String>,
    pub identity_kind: String,
    pub display_name: Option<String>,
    pub address: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug)]
pub struct CanonicalIdentityRecord {
    pub identity_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub provider_identity_id: Option<String>,
    pub identity_kind: String,
    pub display_name: Option<String>,
    pub address: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ConversationReadError {
    #[error("conversation read failed: {0}")]
    Storage(String),
}

#[async_trait::async_trait]
pub trait ConversationReadPort: Send + Sync {
    async fn list_conversations(
        &self,
        account_id: Option<&str>,
        channel_kinds: &[&str],
        title_query: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationRecord>, ConversationReadError>;

    async fn get_conversation(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<CanonicalConversationRecord>, ConversationReadError>;

    async fn list_conversation_members(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
        query: Option<&str>,
        role: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationMemberRecord>, ConversationReadError>;

    async fn get_conversation_from_message_projection(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<CanonicalConversationRecord>, ConversationReadError>;

    async fn list_members_for_provider_conversation(
        &self,
        account_id: &str,
        provider_conversation_id: &str,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationMemberRecord>, ConversationReadError>;

    async fn list_presence(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalPresenceRecord>, ConversationReadError>;

    async fn list_whatsapp_identities(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<CanonicalIdentityRecord>, ConversationReadError>;
}
