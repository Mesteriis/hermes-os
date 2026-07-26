//! Protobuf wire mapping for the public WhatsApp operational query surface.

use prost::Message;

use crate::{
    WhatsAppProviderEvent,
    client_wire::{
        ClientWireError, dialog_to_wire, event_kind_from_wire, event_kind_to_wire, event_to_wire,
        message_to_wire, parse_dialog, parse_event, parse_message, parse_participant,
        participant_to_wire,
    },
    operational::{
        WhatsAppOperationalPageV1, WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryV1,
        WhatsAppOperationalRuntimeStatusV1, validate_operational_query,
    },
    operational_wire_generated as wire,
};

pub fn encode_operational_query(
    query: &WhatsAppOperationalQueryV1,
) -> Result<Vec<u8>, ClientWireError> {
    validate_operational_query(query).map_err(|_| ClientWireError::InvalidPayload)?;
    use wire::whats_app_operational_query_v1::Query;
    let query = match query {
        WhatsAppOperationalQueryV1::ListMessages {
            account_id,
            provider_chat_id,
            cursor,
            limit,
        } => Query::ListMessages(wire::ListMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        WhatsAppOperationalQueryV1::SearchMessages {
            account_id,
            provider_chat_id,
            query,
            cursor,
            limit,
        } => Query::SearchMessages(wire::SearchMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            query: query.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        WhatsAppOperationalQueryV1::ListDialogs {
            account_id,
            cursor,
            limit,
        } => Query::ListDialogs(wire::ListDialogsQuery {
            account_id: account_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        WhatsAppOperationalQueryV1::ListParticipants {
            account_id,
            provider_chat_id,
            cursor,
            limit,
        } => Query::ListParticipants(wire::ListParticipantsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        WhatsAppOperationalQueryV1::ListEvents {
            account_id,
            kind,
            provider_chat_id,
            cursor,
            limit,
        } => Query::ListEvents(wire::ListEventsQuery {
            account_id: account_id.clone(),
            kind: kind.map(event_kind_to_wire),
            provider_chat_id: provider_chat_id.clone(),
            cursor: cursor.clone(),
            limit: *limit,
        }),
        WhatsAppOperationalQueryV1::GetRuntimeStatus { account_id } => {
            Query::GetRuntimeStatus(wire::GetRuntimeStatusQuery {
                account_id: account_id.clone(),
            })
        }
    };
    Ok(wire::WhatsAppOperationalQueryV1 { query: Some(query) }.encode_to_vec())
}

pub fn decode_operational_query(
    bytes: &[u8],
) -> Result<WhatsAppOperationalQueryV1, ClientWireError> {
    use wire::whats_app_operational_query_v1::Query;
    let query = wire::WhatsAppOperationalQueryV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?
        .query
        .ok_or(ClientWireError::MissingVariant)?;
    let query = match query {
        Query::ListMessages(value) => WhatsAppOperationalQueryV1::ListMessages {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::SearchMessages(value) => WhatsAppOperationalQueryV1::SearchMessages {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            query: value.query,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListDialogs(value) => WhatsAppOperationalQueryV1::ListDialogs {
            account_id: value.account_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListParticipants(value) => WhatsAppOperationalQueryV1::ListParticipants {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::ListEvents(value) => WhatsAppOperationalQueryV1::ListEvents {
            account_id: value.account_id,
            kind: value.kind.map(event_kind_from_wire).transpose()?,
            provider_chat_id: value.provider_chat_id,
            cursor: value.cursor,
            limit: value.limit,
        },
        Query::GetRuntimeStatus(value) => WhatsAppOperationalQueryV1::GetRuntimeStatus {
            account_id: value.account_id,
        },
    };
    validate_operational_query(&query).map_err(|_| ClientWireError::InvalidPayload)?;
    Ok(query)
}

#[must_use]
pub fn encode_operational_query_response(response: &WhatsAppOperationalQueryResponseV1) -> Vec<u8> {
    use wire::whats_app_operational_query_response_v1::Response;
    let response = match response {
        WhatsAppOperationalQueryResponseV1::Messages(page) => {
            Response::Messages(wire::WhatsAppMessagePageV1 {
                item: page.items.iter().map(message_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        WhatsAppOperationalQueryResponseV1::Dialogs(page) => {
            Response::Dialogs(wire::WhatsAppDialogPageV1 {
                item: page.items.iter().map(dialog_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        WhatsAppOperationalQueryResponseV1::Participants(page) => {
            Response::Participants(wire::WhatsAppParticipantPageV1 {
                item: page.items.iter().map(participant_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        WhatsAppOperationalQueryResponseV1::Events(page) => {
            Response::Events(wire::WhatsAppEventPageV1 {
                item: page.items.iter().map(event_to_wire).collect(),
                next_cursor: page.next_cursor.clone(),
            })
        }
        WhatsAppOperationalQueryResponseV1::RuntimeStatus(status) => {
            Response::RuntimeStatus(wire::WhatsAppOperationalRuntimeStatusV1 {
                account_id: status.account_id.clone(),
                runtime_state: status.runtime_state.clone(),
                projection_ready: status.projection_ready,
                latest_event_sequence: status.latest_event_sequence,
            })
        }
    };
    wire::WhatsAppOperationalQueryResponseV1 {
        response: Some(response),
    }
    .encode_to_vec()
}

pub fn decode_operational_query_response(
    bytes: &[u8],
) -> Result<WhatsAppOperationalQueryResponseV1, ClientWireError> {
    use wire::whats_app_operational_query_response_v1::Response;
    let response = wire::WhatsAppOperationalQueryResponseV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?
        .response
        .ok_or(ClientWireError::MissingVariant)?;
    Ok(match response {
        Response::Messages(page) => {
            WhatsAppOperationalQueryResponseV1::Messages(WhatsAppOperationalPageV1 {
                items: page.item.into_iter().map(parse_message).collect(),
                next_cursor: page.next_cursor,
            })
        }
        Response::Dialogs(page) => {
            WhatsAppOperationalQueryResponseV1::Dialogs(WhatsAppOperationalPageV1 {
                items: page.item.into_iter().map(parse_dialog).collect(),
                next_cursor: page.next_cursor,
            })
        }
        Response::Participants(page) => {
            WhatsAppOperationalQueryResponseV1::Participants(WhatsAppOperationalPageV1 {
                items: page.item.into_iter().map(parse_participant).collect(),
                next_cursor: page.next_cursor,
            })
        }
        Response::Events(page) => {
            WhatsAppOperationalQueryResponseV1::Events(WhatsAppOperationalPageV1 {
                items: page
                    .item
                    .into_iter()
                    .map(parse_event)
                    .collect::<Result<Vec<_>, _>>()?,
                next_cursor: page.next_cursor,
            })
        }
        Response::RuntimeStatus(status) => {
            WhatsAppOperationalQueryResponseV1::RuntimeStatus(WhatsAppOperationalRuntimeStatusV1 {
                account_id: status.account_id,
                runtime_state: status.runtime_state,
                projection_ready: status.projection_ready,
                latest_event_sequence: status.latest_event_sequence,
            })
        }
    })
}

#[must_use]
pub fn encode_provider_event(event: &WhatsAppProviderEvent) -> Vec<u8> {
    event_to_wire(event).encode_to_vec()
}

pub fn decode_provider_event(bytes: &[u8]) -> Result<WhatsAppProviderEvent, ClientWireError> {
    parse_event(
        crate::wire::WhatsAppProviderEventV1::decode(bytes)
            .map_err(|_| ClientWireError::InvalidPayload)?,
    )
}

#[cfg(test)]
mod tests {
    use crate::{WhatsAppMessage, WhatsAppProviderEvent};

    use super::*;

    #[test]
    fn operational_query_round_trips_without_private_host_shape() {
        let query = WhatsAppOperationalQueryV1::SearchMessages {
            account_id: "account-1".to_owned(),
            provider_chat_id: Some("chat-1".to_owned()),
            query: "decision".to_owned(),
            cursor: Some("v1.messages.scope.12".to_owned()),
            limit: 20,
        };
        let encoded = encode_operational_query(&query).expect("encode operational query");
        assert_eq!(
            decode_operational_query(&encoded).expect("decode operational query"),
            query
        );
    }

    #[test]
    fn operational_message_page_and_event_round_trip() {
        let message = WhatsAppMessage {
            account_id: "account-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: "message-1".to_owned(),
            sender_id: "sender-1".to_owned(),
            sender_display_name: "Sender".to_owned(),
            text: Some("body".to_owned()),
            reply_to_provider_message_id: None,
            occurred_at_unix_seconds: 1_700_000_000,
            delivery_state: Some("delivered".to_owned()),
        };
        let response = WhatsAppOperationalQueryResponseV1::Messages(WhatsAppOperationalPageV1 {
            items: vec![message.clone()],
            next_cursor: Some("v1.messages.scope.1".to_owned()),
        });
        let encoded = encode_operational_query_response(&response);
        assert_eq!(
            decode_operational_query_response(&encoded).expect("decode operational response"),
            response
        );

        let event = WhatsAppProviderEvent::MessageObserved(message);
        assert_eq!(
            decode_provider_event(&encode_provider_event(&event)).expect("decode event"),
            event
        );
    }
}
