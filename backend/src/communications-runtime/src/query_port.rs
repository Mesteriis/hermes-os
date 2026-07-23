//! Generated Communications metadata-query port.

use hermes_communications_api::{
    CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationSourceCursorV1,
    GetCommunicationConversationV1, ListCommunicationAccountsV1,
    ListCommunicationConversationsV1, ListConversationMessagesV1,
    ListConversationParticipantsV1, ListMessageAttachmentAnchorsV1,
    ListMessageReferencesV1,
    query_wire::{
        CommunicationsQueryRequestV1, CommunicationsQueryResponseV1,
        communications_query_request_v1::Operation,
        communications_query_response_v1::Result as QueryResult,
    },
};
use hermes_communications_persistence::CommunicationsDurablePersistence;
use prost::Message;

use crate::query::{
    get_communication_conversation, list_communication_accounts,
    list_communication_conversations, list_conversation_messages,
    list_conversation_participants, list_message_attachment_anchors,
    list_message_references,
};

const PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsQueryPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_query_request_v1(
    persistence: &CommunicationsDurablePersistence,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsQueryPortErrorV1> {
    let request = CommunicationsQueryRequestV1::decode(bytes)
        .map_err(|_| CommunicationsQueryPortErrorV1::Protocol)?;
    if request.protocol_major != PROTOCOL_MAJOR {
        return Err(CommunicationsQueryPortErrorV1::Protocol);
    }
    let result = match request.operation.ok_or(CommunicationsQueryPortErrorV1::Protocol)? {
        Operation::ListAccounts(request) => QueryResult::ListAccounts(
            hermes_communications_api::query_wire::ListAccountsResponseV1 {
                accounts: list_communication_accounts(
                    persistence,
                    ListCommunicationAccountsV1 { limit: limit(request.limit)? },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        Operation::ListConversations(request) => QueryResult::ListConversations(
            hermes_communications_api::query_wire::ListConversationsResponseV1 {
                conversations: list_communication_conversations(
                    persistence,
                    ListCommunicationConversationsV1 {
                        account_cursor: optional_cursor(&request.account_cursor_sha256)?,
                        limit: limit(request.limit)?,
                    },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        Operation::GetConversation(request) => QueryResult::GetConversation(
            hermes_communications_api::query_wire::GetConversationResponseV1 {
                conversation: Some(
                    (&get_communication_conversation(
                        persistence,
                        GetCommunicationConversationV1 {
                            conversation_id: CommunicationConversationIdV1::new(id16(&request.conversation_id)?),
                        },
                    )
                    .await
                    .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?)
                        .into(),
                ),
            },
        ),
        Operation::ListConversationMessages(request) => QueryResult::ListConversationMessages(
            hermes_communications_api::query_wire::ListConversationMessagesResponseV1 {
                messages: list_conversation_messages(
                    persistence,
                    ListConversationMessagesV1 {
                        conversation_id: CommunicationConversationIdV1::new(id16(&request.conversation_id)?),
                        limit: limit(request.limit)?,
                    },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        Operation::ListConversationParticipants(request) => QueryResult::ListConversationParticipants(
            hermes_communications_api::query_wire::ListConversationParticipantsResponseV1 {
                participants: list_conversation_participants(
                    persistence,
                    ListConversationParticipantsV1 {
                        conversation_id: CommunicationConversationIdV1::new(id16(&request.conversation_id)?),
                        limit: limit(request.limit)?,
                    },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        Operation::ListMessageAttachmentAnchors(request) => QueryResult::ListMessageAttachmentAnchors(
            hermes_communications_api::query_wire::ListMessageAttachmentAnchorsResponseV1 {
                anchors: list_message_attachment_anchors(
                    persistence,
                    ListMessageAttachmentAnchorsV1 {
                        message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                        limit: limit(request.limit)?,
                    },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        Operation::ListMessageReferences(request) => QueryResult::ListMessageReferences(
            hermes_communications_api::query_wire::ListMessageReferencesResponseV1 {
                references: list_message_references(
                    persistence,
                    ListMessageReferencesV1 {
                        message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                        limit: limit(request.limit)?,
                    },
                )
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .iter()
                .map(Into::into)
                .collect(),
            },
        ),
        // Search remains unavailable until the managed runtime has both the
        // owner-derived key lease and bounded Blob reader. Returning an empty
        // result here would incorrectly claim that canonical evidence has no
        // matches.
        Operation::SearchCommunications(_) => return Err(CommunicationsQueryPortErrorV1::Unavailable),
    };
    Ok(CommunicationsQueryResponseV1 {
        result: Some(result),
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn limit(value: u32) -> Result<u16, CommunicationsQueryPortErrorV1> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(CommunicationsQueryPortErrorV1::Protocol)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsQueryPortErrorV1> {
    value.try_into().map_err(|_| CommunicationsQueryPortErrorV1::Protocol)
}

fn optional_cursor(
    value: &[u8],
) -> Result<Option<CommunicationSourceCursorV1>, CommunicationsQueryPortErrorV1> {
    if value.is_empty() {
        return Ok(None);
    }
    let cursor: [u8; 32] = value.try_into().map_err(|_| CommunicationsQueryPortErrorV1::Protocol)?;
    Ok(Some(CommunicationSourceCursorV1::new(cursor)))
}
