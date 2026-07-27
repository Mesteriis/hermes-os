//! Public Zulip-owned operational read contract.

use crate::ZulipAttachmentV1;

pub const MAX_OPERATIONAL_CURSOR_BYTES: usize = 512;
pub const MAX_OPERATIONAL_PAGE_SIZE: u32 = 200;
pub const MAX_OPERATIONAL_QUERY_BYTES: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipConversationKindV1 {
    StreamTopic,
    Direct,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipConversationV1 {
    pub account_id: String,
    pub provider_conversation_id: String,
    pub kind: ZulipConversationKindV1,
    pub stream_id: Option<String>,
    pub stream_name: Option<String>,
    pub topic: Option<String>,
    pub direct_recipient_id: Option<String>,
    pub latest_provider_message_id: Option<String>,
    pub latest_event_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipReactionStateV1 {
    pub actor_id: String,
    pub emoji_name: String,
    pub emoji_code: Option<String>,
    pub reaction_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipMessageV1 {
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_conversation_id: String,
    pub sender_id: String,
    pub is_outgoing: bool,
    pub content: Option<String>,
    pub sent_at_unix_seconds: Option<i64>,
    pub edited_at_unix_seconds: Option<i64>,
    pub deleted: bool,
    pub attachments: Vec<ZulipAttachmentV1>,
    pub reactions: Vec<ZulipReactionStateV1>,
    pub last_event_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipOperationalEventKindV1 {
    MessageUpserted,
    MessageUpdated,
    MessageDeleted,
    ReactionAdded,
    ReactionRemoved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipOperationalEventV1 {
    pub account_id: String,
    pub provider_event_id: i64,
    pub provider_message_id: String,
    pub provider_conversation_id: Option<String>,
    pub actor_id: Option<String>,
    pub kind: ZulipOperationalEventKindV1,
    pub content: Option<String>,
    pub topic: Option<String>,
    pub reaction: Option<ZulipReactionStateV1>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipHistoryStateV1 {
    NotStarted,
    Syncing,
    Ready,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipAccountStatusV1 {
    pub account_id: String,
    pub projection_ready: bool,
    pub history_state: ZulipHistoryStateV1,
    pub oldest_provider_message_id: Option<String>,
    pub last_provider_event_id: Option<i64>,
    pub latest_event_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipOperationalQueryV1 {
    ListMessages {
        account_id: String,
        provider_conversation_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    SearchMessages {
        account_id: String,
        provider_conversation_id: Option<String>,
        query: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListConversations {
        account_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListEvents {
        account_id: String,
        kind: Option<ZulipOperationalEventKindV1>,
        provider_conversation_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    GetAccountStatus {
        account_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipOperationalPageV1<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipOperationalQueryResponseV1 {
    Messages(ZulipOperationalPageV1<ZulipMessageV1>),
    Conversations(ZulipOperationalPageV1<ZulipConversationV1>),
    Events(ZulipOperationalPageV1<ZulipOperationalEventV1>),
    AccountStatus(ZulipAccountStatusV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipOperationalContractErrorV1 {
    InvalidId,
    InvalidCursor,
    InvalidLimit,
    InvalidQuery,
}

#[must_use]
pub fn operational_query_account_id(query: &ZulipOperationalQueryV1) -> &str {
    match query {
        ZulipOperationalQueryV1::ListMessages { account_id, .. }
        | ZulipOperationalQueryV1::SearchMessages { account_id, .. }
        | ZulipOperationalQueryV1::ListConversations { account_id, .. }
        | ZulipOperationalQueryV1::ListEvents { account_id, .. }
        | ZulipOperationalQueryV1::GetAccountStatus { account_id } => account_id,
    }
}

pub fn validate_operational_query(
    query: &ZulipOperationalQueryV1,
) -> Result<(), ZulipOperationalContractErrorV1> {
    validate_id(operational_query_account_id(query))?;
    match query {
        ZulipOperationalQueryV1::GetAccountStatus { .. } => Ok(()),
        ZulipOperationalQueryV1::ListMessages {
            provider_conversation_id,
            cursor,
            limit,
            ..
        }
        | ZulipOperationalQueryV1::ListEvents {
            provider_conversation_id,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(provider_conversation_id.as_deref())?;
            validate_page(cursor.as_deref(), *limit)
        }
        ZulipOperationalQueryV1::SearchMessages {
            provider_conversation_id,
            query,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(provider_conversation_id.as_deref())?;
            if query.trim().is_empty()
                || query.len() > MAX_OPERATIONAL_QUERY_BYTES
                || query.contains('\0')
            {
                return Err(ZulipOperationalContractErrorV1::InvalidQuery);
            }
            validate_page(cursor.as_deref(), *limit)
        }
        ZulipOperationalQueryV1::ListConversations { cursor, limit, .. } => {
            validate_page(cursor.as_deref(), *limit)
        }
    }
}

fn validate_optional_id(value: Option<&str>) -> Result<(), ZulipOperationalContractErrorV1> {
    if let Some(value) = value {
        validate_id(value)?;
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), ZulipOperationalContractErrorV1> {
    if value.trim().is_empty() || value.len() > 512 || value.contains(['\0', '\r', '\n']) {
        return Err(ZulipOperationalContractErrorV1::InvalidId);
    }
    Ok(())
}

fn validate_page(cursor: Option<&str>, limit: u32) -> Result<(), ZulipOperationalContractErrorV1> {
    if limit == 0 || limit > MAX_OPERATIONAL_PAGE_SIZE {
        return Err(ZulipOperationalContractErrorV1::InvalidLimit);
    }
    if let Some(cursor) = cursor
        && (cursor.is_empty()
            || cursor.len() > MAX_OPERATIONAL_CURSOR_BYTES
            || !cursor.is_ascii()
            || cursor.chars().any(char::is_whitespace))
    {
        return Err(ZulipOperationalContractErrorV1::InvalidCursor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_validation_keeps_content_and_cursor_bounded() {
        assert_eq!(
            validate_operational_query(&ZulipOperationalQueryV1::SearchMessages {
                account_id: "account".into(),
                provider_conversation_id: Some("stream:1:topic".into()),
                query: "decision".into(),
                cursor: Some("z1m.scope.42".into()),
                limit: MAX_OPERATIONAL_PAGE_SIZE,
            }),
            Ok(())
        );
        assert_eq!(
            validate_operational_query(&ZulipOperationalQueryV1::ListConversations {
                account_id: "account".into(),
                cursor: None,
                limit: MAX_OPERATIONAL_PAGE_SIZE + 1,
            }),
            Err(ZulipOperationalContractErrorV1::InvalidLimit)
        );
    }
}
