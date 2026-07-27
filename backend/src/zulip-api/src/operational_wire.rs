//! Protobuf mapping for the public Zulip operational query surface.

use prost::Message;

use crate::{
    ZulipAttachmentV1,
    client_wire::ZulipClientWireErrorV1,
    operational::{
        ZulipAccountStatusV1, ZulipConversationKindV1, ZulipConversationV1, ZulipHistoryStateV1,
        ZulipMessageV1, ZulipOperationalEventKindV1, ZulipOperationalEventV1,
        ZulipOperationalPageV1, ZulipOperationalQueryResponseV1, ZulipOperationalQueryV1,
        ZulipReactionStateV1, validate_operational_query,
    },
    operational_wire_generated as wire,
};

pub fn encode_operational_query(
    query: &ZulipOperationalQueryV1,
) -> Result<Vec<u8>, ZulipClientWireErrorV1> {
    validate_operational_query(query).map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    use wire::zulip_operational_query_v1::Query;
    let query = match query {
        ZulipOperationalQueryV1::ListMessages {
            account_id,
            provider_conversation_id,
            cursor,
            limit,
        } => Query::ListMessages(wire::ListMessagesQuery {
            account_id: account_id.clone(),
            provider_conversation_id: provider_conversation_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        ZulipOperationalQueryV1::SearchMessages {
            account_id,
            provider_conversation_id,
            query,
            cursor,
            limit,
        } => Query::SearchMessages(wire::SearchMessagesQuery {
            account_id: account_id.clone(),
            provider_conversation_id: provider_conversation_id.clone(),
            query: query.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        ZulipOperationalQueryV1::ListConversations {
            account_id,
            cursor,
            limit,
        } => Query::ListConversations(wire::ListConversationsQuery {
            account_id: account_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        ZulipOperationalQueryV1::ListEvents {
            account_id,
            kind,
            provider_conversation_id,
            cursor,
            limit,
        } => Query::ListEvents(wire::ListEventsQuery {
            account_id: account_id.clone(),
            kind: kind.map(event_kind_to_wire),
            provider_conversation_id: provider_conversation_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        ZulipOperationalQueryV1::GetAccountStatus { account_id } => {
            Query::GetAccountStatus(wire::GetAccountStatusQuery {
                account_id: account_id.clone(),
            })
        }
    };
    Ok(wire::ZulipOperationalQueryV1 { query: Some(query) }.encode_to_vec())
}

pub fn decode_operational_query(
    bytes: &[u8],
) -> Result<ZulipOperationalQueryV1, ZulipClientWireErrorV1> {
    use wire::zulip_operational_query_v1::Query;
    let query = wire::ZulipOperationalQueryV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?
        .query
        .ok_or(ZulipClientWireErrorV1::MissingVariant)?;
    let query = match query {
        Query::ListMessages(value) => ZulipOperationalQueryV1::ListMessages {
            account_id: value.account_id,
            provider_conversation_id: value.provider_conversation_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::SearchMessages(value) => ZulipOperationalQueryV1::SearchMessages {
            account_id: value.account_id,
            provider_conversation_id: value.provider_conversation_id,
            query: value.query,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListConversations(value) => ZulipOperationalQueryV1::ListConversations {
            account_id: value.account_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListEvents(value) => ZulipOperationalQueryV1::ListEvents {
            account_id: value.account_id,
            kind: value.kind.map(event_kind_from_wire).transpose()?,
            provider_conversation_id: value.provider_conversation_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetAccountStatus(value) => ZulipOperationalQueryV1::GetAccountStatus {
            account_id: value.account_id,
        },
    };
    validate_operational_query(&query).map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(query)
}

#[must_use]
pub fn encode_operational_query_response(response: &ZulipOperationalQueryResponseV1) -> Vec<u8> {
    use wire::zulip_operational_query_response_v1::Response;
    let response = match response {
        ZulipOperationalQueryResponseV1::Messages(page) => {
            Response::Messages(wire::ZulipMessagePageV1 {
                item: page.items.iter().map(message_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        ZulipOperationalQueryResponseV1::Conversations(page) => {
            Response::Conversations(wire::ZulipConversationPageV1 {
                item: page.items.iter().map(conversation_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        ZulipOperationalQueryResponseV1::Events(page) => Response::Events(wire::ZulipEventPageV1 {
            item: page.items.iter().map(event_to_wire).collect(),
            next_cursor: page.next_cursor.clone(),
        }),
        ZulipOperationalQueryResponseV1::AccountStatus(status) => {
            Response::AccountStatus(status_to_wire(status))
        }
    };
    wire::ZulipOperationalQueryResponseV1 {
        response: Some(response),
    }
    .encode_to_vec()
}

pub fn decode_operational_query_response(
    bytes: &[u8],
) -> Result<ZulipOperationalQueryResponseV1, ZulipClientWireErrorV1> {
    use wire::zulip_operational_query_response_v1::Response;
    let response = wire::ZulipOperationalQueryResponseV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?
        .response
        .ok_or(ZulipClientWireErrorV1::MissingVariant)?;
    Ok(match response {
        Response::Messages(page) => {
            ZulipOperationalQueryResponseV1::Messages(ZulipOperationalPageV1 {
                items: page
                    .item
                    .into_iter()
                    .map(message_from_wire)
                    .collect::<Result<_, _>>()?,
                next_cursor: page.next_cursor,
            })
        }
        Response::Conversations(page) => {
            ZulipOperationalQueryResponseV1::Conversations(ZulipOperationalPageV1 {
                items: page
                    .item
                    .into_iter()
                    .map(conversation_from_wire)
                    .collect::<Result<_, _>>()?,
                next_cursor: page.next_cursor,
            })
        }
        Response::Events(page) => ZulipOperationalQueryResponseV1::Events(ZulipOperationalPageV1 {
            items: page
                .item
                .into_iter()
                .map(event_from_wire)
                .collect::<Result<_, _>>()?,
            next_cursor: page.next_cursor,
        }),
        Response::AccountStatus(status) => {
            ZulipOperationalQueryResponseV1::AccountStatus(status_from_wire(status)?)
        }
    })
}

#[must_use]
pub fn encode_operational_event(event: &ZulipOperationalEventV1) -> Vec<u8> {
    event_to_wire(event).encode_to_vec()
}

pub fn decode_operational_event(
    bytes: &[u8],
) -> Result<ZulipOperationalEventV1, ZulipClientWireErrorV1> {
    event_from_wire(
        wire::ZulipOperationalEventV1::decode(bytes)
            .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?,
    )
}

fn message_to_wire(message: &ZulipMessageV1) -> wire::ZulipMessageV1 {
    wire::ZulipMessageV1 {
        account_id: message.account_id.clone(),
        provider_message_id: message.provider_message_id.clone(),
        provider_conversation_id: message.provider_conversation_id.clone(),
        sender_id: message.sender_id.clone(),
        is_outgoing: message.is_outgoing,
        content: message.content.clone(),
        sent_at_unix_seconds: message.sent_at_unix_seconds,
        edited_at_unix_seconds: message.edited_at_unix_seconds,
        deleted: message.deleted,
        attachment: message.attachments.iter().map(attachment_to_wire).collect(),
        reaction: message.reactions.iter().map(reaction_to_wire).collect(),
        last_event_sequence: message.last_event_sequence,
    }
}

fn message_from_wire(
    message: wire::ZulipMessageV1,
) -> Result<ZulipMessageV1, ZulipClientWireErrorV1> {
    validate_required(&message.account_id)?;
    validate_required(&message.provider_message_id)?;
    validate_required(&message.provider_conversation_id)?;
    validate_required(&message.sender_id)?;
    Ok(ZulipMessageV1 {
        account_id: message.account_id,
        provider_message_id: message.provider_message_id,
        provider_conversation_id: message.provider_conversation_id,
        sender_id: message.sender_id,
        is_outgoing: message.is_outgoing,
        content: message.content,
        sent_at_unix_seconds: message.sent_at_unix_seconds,
        edited_at_unix_seconds: message.edited_at_unix_seconds,
        deleted: message.deleted,
        attachments: message
            .attachment
            .into_iter()
            .map(attachment_from_wire)
            .collect::<Result<_, _>>()?,
        reactions: message
            .reaction
            .into_iter()
            .map(reaction_from_wire)
            .collect::<Result<_, _>>()?,
        last_event_sequence: message.last_event_sequence,
    })
}

fn conversation_to_wire(conversation: &ZulipConversationV1) -> wire::ZulipConversationV1 {
    wire::ZulipConversationV1 {
        account_id: conversation.account_id.clone(),
        provider_conversation_id: conversation.provider_conversation_id.clone(),
        kind: conversation_kind_to_wire(conversation.kind),
        stream_id: conversation.stream_id.clone(),
        stream_name: conversation.stream_name.clone(),
        topic: conversation.topic.clone(),
        direct_recipient_id: conversation.direct_recipient_id.clone(),
        latest_provider_message_id: conversation.latest_provider_message_id.clone(),
        latest_event_sequence: conversation.latest_event_sequence,
    }
}

fn conversation_from_wire(
    conversation: wire::ZulipConversationV1,
) -> Result<ZulipConversationV1, ZulipClientWireErrorV1> {
    validate_required(&conversation.account_id)?;
    validate_required(&conversation.provider_conversation_id)?;
    Ok(ZulipConversationV1 {
        account_id: conversation.account_id,
        provider_conversation_id: conversation.provider_conversation_id,
        kind: conversation_kind_from_wire(conversation.kind)?,
        stream_id: conversation.stream_id,
        stream_name: conversation.stream_name,
        topic: conversation.topic,
        direct_recipient_id: conversation.direct_recipient_id,
        latest_provider_message_id: conversation.latest_provider_message_id,
        latest_event_sequence: conversation.latest_event_sequence,
    })
}

fn event_to_wire(event: &ZulipOperationalEventV1) -> wire::ZulipOperationalEventV1 {
    wire::ZulipOperationalEventV1 {
        account_id: event.account_id.clone(),
        provider_event_id: event.provider_event_id,
        provider_message_id: event.provider_message_id.clone(),
        provider_conversation_id: event.provider_conversation_id.clone(),
        actor_id: event.actor_id.clone(),
        kind: event_kind_to_wire(event.kind),
        content: event.content.clone(),
        topic: event.topic.clone(),
        reaction: event.reaction.as_ref().map(reaction_to_wire),
        observed_at_unix_seconds: event.observed_at_unix_seconds,
    }
}

fn event_from_wire(
    event: wire::ZulipOperationalEventV1,
) -> Result<ZulipOperationalEventV1, ZulipClientWireErrorV1> {
    validate_required(&event.account_id)?;
    validate_required(&event.provider_message_id)?;
    if event.provider_event_id <= 0 || event.observed_at_unix_seconds <= 0 {
        return Err(ZulipClientWireErrorV1::InvalidPayload);
    }
    Ok(ZulipOperationalEventV1 {
        account_id: event.account_id,
        provider_event_id: event.provider_event_id,
        provider_message_id: event.provider_message_id,
        provider_conversation_id: event.provider_conversation_id,
        actor_id: event.actor_id,
        kind: event_kind_from_wire(event.kind)?,
        content: event.content,
        topic: event.topic,
        reaction: event.reaction.map(reaction_from_wire).transpose()?,
        observed_at_unix_seconds: event.observed_at_unix_seconds,
    })
}

fn status_to_wire(status: &ZulipAccountStatusV1) -> wire::ZulipAccountStatusV1 {
    wire::ZulipAccountStatusV1 {
        account_id: status.account_id.clone(),
        projection_ready: status.projection_ready,
        history_state: history_state_to_wire(status.history_state),
        oldest_provider_message_id: status.oldest_provider_message_id.clone(),
        last_provider_event_id: status.last_provider_event_id,
        latest_event_sequence: status.latest_event_sequence,
    }
}

fn status_from_wire(
    status: wire::ZulipAccountStatusV1,
) -> Result<ZulipAccountStatusV1, ZulipClientWireErrorV1> {
    validate_required(&status.account_id)?;
    Ok(ZulipAccountStatusV1 {
        account_id: status.account_id,
        projection_ready: status.projection_ready,
        history_state: history_state_from_wire(status.history_state)?,
        oldest_provider_message_id: status.oldest_provider_message_id,
        last_provider_event_id: status.last_provider_event_id,
        latest_event_sequence: status.latest_event_sequence,
    })
}

fn attachment_to_wire(attachment: &ZulipAttachmentV1) -> wire::ZulipAttachmentV1 {
    wire::ZulipAttachmentV1 {
        provider_attachment_id: attachment.provider_attachment_id.clone(),
        filename: attachment.filename.clone(),
    }
}

fn attachment_from_wire(
    attachment: wire::ZulipAttachmentV1,
) -> Result<ZulipAttachmentV1, ZulipClientWireErrorV1> {
    validate_required(&attachment.provider_attachment_id)?;
    Ok(ZulipAttachmentV1 {
        provider_attachment_id: attachment.provider_attachment_id,
        filename: attachment.filename,
    })
}

fn reaction_to_wire(reaction: &ZulipReactionStateV1) -> wire::ZulipReactionStateV1 {
    wire::ZulipReactionStateV1 {
        actor_id: reaction.actor_id.clone(),
        emoji_name: reaction.emoji_name.clone(),
        emoji_code: reaction.emoji_code.clone(),
        reaction_type: reaction.reaction_type.clone(),
    }
}

fn reaction_from_wire(
    reaction: wire::ZulipReactionStateV1,
) -> Result<ZulipReactionStateV1, ZulipClientWireErrorV1> {
    validate_required(&reaction.actor_id)?;
    validate_required(&reaction.emoji_name)?;
    Ok(ZulipReactionStateV1 {
        actor_id: reaction.actor_id,
        emoji_name: reaction.emoji_name,
        emoji_code: reaction.emoji_code,
        reaction_type: reaction.reaction_type,
    })
}

fn validate_required(value: &str) -> Result<(), ZulipClientWireErrorV1> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(ZulipClientWireErrorV1::InvalidPayload);
    }
    Ok(())
}

const fn conversation_kind_to_wire(kind: ZulipConversationKindV1) -> i32 {
    match kind {
        ZulipConversationKindV1::StreamTopic => 1,
        ZulipConversationKindV1::Direct => 2,
    }
}

fn conversation_kind_from_wire(
    kind: i32,
) -> Result<ZulipConversationKindV1, ZulipClientWireErrorV1> {
    match kind {
        1 => Ok(ZulipConversationKindV1::StreamTopic),
        2 => Ok(ZulipConversationKindV1::Direct),
        _ => Err(ZulipClientWireErrorV1::InvalidPayload),
    }
}

pub(crate) const fn event_kind_to_wire(kind: ZulipOperationalEventKindV1) -> i32 {
    match kind {
        ZulipOperationalEventKindV1::MessageUpserted => 1,
        ZulipOperationalEventKindV1::MessageUpdated => 2,
        ZulipOperationalEventKindV1::MessageDeleted => 3,
        ZulipOperationalEventKindV1::ReactionAdded => 4,
        ZulipOperationalEventKindV1::ReactionRemoved => 5,
    }
}

pub(crate) fn event_kind_from_wire(
    kind: i32,
) -> Result<ZulipOperationalEventKindV1, ZulipClientWireErrorV1> {
    match kind {
        1 => Ok(ZulipOperationalEventKindV1::MessageUpserted),
        2 => Ok(ZulipOperationalEventKindV1::MessageUpdated),
        3 => Ok(ZulipOperationalEventKindV1::MessageDeleted),
        4 => Ok(ZulipOperationalEventKindV1::ReactionAdded),
        5 => Ok(ZulipOperationalEventKindV1::ReactionRemoved),
        _ => Err(ZulipClientWireErrorV1::InvalidPayload),
    }
}

const fn history_state_to_wire(state: ZulipHistoryStateV1) -> i32 {
    match state {
        ZulipHistoryStateV1::NotStarted => 1,
        ZulipHistoryStateV1::Syncing => 2,
        ZulipHistoryStateV1::Ready => 3,
        ZulipHistoryStateV1::Degraded => 4,
    }
}

fn history_state_from_wire(state: i32) -> Result<ZulipHistoryStateV1, ZulipClientWireErrorV1> {
    match state {
        1 => Ok(ZulipHistoryStateV1::NotStarted),
        2 => Ok(ZulipHistoryStateV1::Syncing),
        3 => Ok(ZulipHistoryStateV1::Ready),
        4 => Ok(ZulipHistoryStateV1::Degraded),
        _ => Err(ZulipClientWireErrorV1::InvalidPayload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operational_query_and_response_round_trip() {
        let query = ZulipOperationalQueryV1::SearchMessages {
            account_id: "account".into(),
            provider_conversation_id: Some("stream:1:topic".into()),
            query: "decision".into(),
            cursor: Some("z1m.scope.42".into()),
            limit: 20,
        };
        assert_eq!(
            decode_operational_query(&encode_operational_query(&query).expect("encode")),
            Ok(query)
        );

        let response = ZulipOperationalQueryResponseV1::AccountStatus(ZulipAccountStatusV1 {
            account_id: "account".into(),
            projection_ready: true,
            history_state: ZulipHistoryStateV1::Ready,
            oldest_provider_message_id: Some("1".into()),
            last_provider_event_id: Some(9),
            latest_event_sequence: 11,
        });
        assert_eq!(
            decode_operational_query_response(&encode_operational_query_response(&response)),
            Ok(response)
        );
    }
}
