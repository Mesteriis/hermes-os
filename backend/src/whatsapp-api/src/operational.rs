//! Public WhatsApp operational read contract.
//!
//! This surface is provider-owned and intentionally separate from command
//! operation status and the private native-host bridge.

use serde::{Deserialize, Serialize};

use crate::{
    WhatsAppContractError, WhatsAppDialog, WhatsAppMessage, WhatsAppParticipant,
    WhatsAppProviderEvent, WhatsAppProviderEventKind, validate_id, validate_text,
};

pub const MAX_OPERATIONAL_CURSOR_BYTES: usize = 512;
pub const MAX_OPERATIONAL_PAGE_SIZE: u32 = 200;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppOperationalQueryV1 {
    ListMessages {
        account_id: String,
        provider_chat_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    SearchMessages {
        account_id: String,
        provider_chat_id: Option<String>,
        query: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListDialogs {
        account_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListParticipants {
        account_id: String,
        provider_chat_id: String,
        cursor: Option<String>,
        limit: u32,
    },
    ListEvents {
        account_id: String,
        kind: Option<WhatsAppProviderEventKind>,
        provider_chat_id: Option<String>,
        cursor: Option<String>,
        limit: u32,
    },
    GetRuntimeStatus {
        account_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppOperationalPageV1<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppOperationalRuntimeStatusV1 {
    pub account_id: String,
    pub runtime_state: Option<String>,
    pub projection_ready: bool,
    pub latest_event_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppOperationalQueryResponseV1 {
    Messages(WhatsAppOperationalPageV1<WhatsAppMessage>),
    Dialogs(WhatsAppOperationalPageV1<WhatsAppDialog>),
    Participants(WhatsAppOperationalPageV1<WhatsAppParticipant>),
    Events(WhatsAppOperationalPageV1<WhatsAppProviderEvent>),
    RuntimeStatus(WhatsAppOperationalRuntimeStatusV1),
}

#[must_use]
pub fn operational_query_account_id(query: &WhatsAppOperationalQueryV1) -> &str {
    match query {
        WhatsAppOperationalQueryV1::ListMessages { account_id, .. }
        | WhatsAppOperationalQueryV1::SearchMessages { account_id, .. }
        | WhatsAppOperationalQueryV1::ListDialogs { account_id, .. }
        | WhatsAppOperationalQueryV1::ListParticipants { account_id, .. }
        | WhatsAppOperationalQueryV1::ListEvents { account_id, .. }
        | WhatsAppOperationalQueryV1::GetRuntimeStatus { account_id } => account_id,
    }
}

pub fn validate_operational_query(
    query: &WhatsAppOperationalQueryV1,
) -> Result<(), WhatsAppContractError> {
    validate_id(operational_query_account_id(query))?;
    match query {
        WhatsAppOperationalQueryV1::GetRuntimeStatus { .. } => Ok(()),
        WhatsAppOperationalQueryV1::ListMessages {
            provider_chat_id,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(provider_chat_id.as_deref())?;
            validate_page(cursor.as_deref(), *limit)
        }
        WhatsAppOperationalQueryV1::SearchMessages {
            provider_chat_id,
            query,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(provider_chat_id.as_deref())?;
            validate_text(query)?;
            validate_page(cursor.as_deref(), *limit)
        }
        WhatsAppOperationalQueryV1::ListDialogs { cursor, limit, .. } => {
            validate_page(cursor.as_deref(), *limit)
        }
        WhatsAppOperationalQueryV1::ListParticipants {
            provider_chat_id,
            cursor,
            limit,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_page(cursor.as_deref(), *limit)
        }
        WhatsAppOperationalQueryV1::ListEvents {
            provider_chat_id,
            cursor,
            limit,
            ..
        } => {
            validate_optional_id(provider_chat_id.as_deref())?;
            validate_page(cursor.as_deref(), *limit)
        }
    }
}

fn validate_optional_id(value: Option<&str>) -> Result<(), WhatsAppContractError> {
    if let Some(value) = value {
        validate_id(value)?;
    }
    Ok(())
}

fn validate_page(cursor: Option<&str>, limit: u32) -> Result<(), WhatsAppContractError> {
    if limit == 0 || limit > MAX_OPERATIONAL_PAGE_SIZE {
        return Err(WhatsAppContractError::FieldTooLong);
    }
    if let Some(cursor) = cursor
        && (cursor.is_empty()
            || cursor.len() > MAX_OPERATIONAL_CURSOR_BYTES
            || !cursor.is_ascii()
            || cursor.chars().any(char::is_whitespace))
    {
        return Err(WhatsAppContractError::InvalidText);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_validation_keeps_cursor_and_page_bounded() {
        let query = WhatsAppOperationalQueryV1::ListMessages {
            account_id: "account-1".to_owned(),
            provider_chat_id: Some("chat-1".to_owned()),
            cursor: Some("v1.messages.scope.42".to_owned()),
            limit: MAX_OPERATIONAL_PAGE_SIZE,
        };
        assert_eq!(validate_operational_query(&query), Ok(()));

        let too_large = WhatsAppOperationalQueryV1::ListDialogs {
            account_id: "account-1".to_owned(),
            cursor: None,
            limit: MAX_OPERATIONAL_PAGE_SIZE + 1,
        };
        assert_eq!(
            validate_operational_query(&too_large),
            Err(WhatsAppContractError::FieldTooLong)
        );
    }

    #[test]
    fn query_validation_rejects_cross_transport_cursor_content() {
        let query = WhatsAppOperationalQueryV1::ListDialogs {
            account_id: "account-1".to_owned(),
            cursor: Some("cursor with whitespace".to_owned()),
            limit: 10,
        };
        assert_eq!(
            validate_operational_query(&query),
            Err(WhatsAppContractError::InvalidText)
        );
    }
}
