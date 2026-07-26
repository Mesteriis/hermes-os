//! Versioned private WhatsApp host/WebView bridge contract.
//!
//! The host may submit only sanitized typed provider observations. It cannot
//! submit credentials, browser state, arbitrary page payloads, or media bytes.
//! Provider commands flow in the opposite direction through an exact bounded
//! lease.

use crate::{
    WhatsAppDialog, WhatsAppMessage, WhatsAppParticipant, WhatsAppProviderCommand,
    WhatsAppProviderEvent, client_wire, validate_event, wire,
};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const HOST_BRIDGE_CONTRACT_NAME: &str = "whatsapp.host_bridge.v1";
pub const HOST_BRIDGE_CONTRACT_MAJOR: u32 = 1;
pub const HOST_BRIDGE_CONTRACT_REVISION: u32 = 1;
pub const HOST_BRIDGE_PROTOCOL_MAJOR: u32 = 1;
pub const HOST_BRIDGE_PROTOCOL_REVISION: u32 = 6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppHostBridgeEnvelopeV1 {
    pub protocol_major: u32,
    pub protocol_revision: u32,
    pub account_id: String,
    pub provider_event_id: String,
    pub observed_at_unix_seconds: i64,
    pub observation: WhatsAppHostObservationV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppHostBridgeHandshakeV1 {
    pub protocol_major: u32,
    pub protocol_revision: u32,
    pub route_binding_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppHostCommandClaimV1 {
    pub account_id: String,
    pub host_claim_id: String,
    pub lease_seconds: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppHostBridgeOperationV1 {
    Observation(Box<WhatsAppHostBridgeEnvelopeV1>),
    ClaimCommands(WhatsAppHostCommandClaimV1),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppHostObservationV1 {
    RuntimeState {
        state: String,
    },
    MessageIdentity {
        provider_chat_id: String,
        provider_message_id: String,
        sender_id: String,
    },
    MessageUpdated {
        provider_chat_id: String,
        provider_message_id: String,
    },
    MessageDeleted {
        provider_chat_id: String,
        provider_message_id: String,
    },
    Receipt {
        provider_chat_id: String,
        provider_message_id: String,
        delivery_state: String,
    },
    Reaction {
        provider_chat_id: String,
        provider_message_id: String,
        actor_id: String,
        emoji: Option<String>,
        is_active: bool,
    },
    Dialog {
        provider_chat_id: String,
        title: String,
        kind: String,
    },
    Participant {
        provider_chat_id: String,
        provider_identity_id: String,
        display_name: String,
    },
    Presence {
        provider_chat_id: String,
        provider_identity_id: String,
        state: String,
    },
    MediaMetadata {
        provider_chat_id: String,
        provider_message_id: String,
        provider_media_id: String,
        media_kind: String,
        filename: Option<String>,
        content_type: Option<String>,
        declared_size: Option<u64>,
    },
    CallMetadata {
        provider_call_id: String,
        provider_chat_id: String,
        direction: String,
        state: String,
    },
    StatusMetadata {
        provider_status_id: String,
        sender_id: String,
    },
    StatusViewMetadata {
        provider_status_id: String,
        viewer_id: String,
    },
    StatusDeletedMetadata {
        provider_status_id: String,
    },
    SessionLinked {
        secret_ref: String,
        revision: u64,
    },
    SessionRevoked,
    CommandResult {
        operation_id: String,
        provider_request_id: Option<String>,
        succeeded: bool,
        host_claim_id: String,
    },
    OperationalMessage(WhatsAppMessage),
    OperationalDialog(WhatsAppDialog),
    OperationalParticipant(WhatsAppParticipant),
    OperationalParticipantRemoved {
        provider_chat_id: String,
        provider_identity_id: String,
    },
    OperationalResyncState {
        complete: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppHostBridgeError {
    InvalidProtocol,
    EmptyField,
    InvalidTimestamp,
    ForbiddenContent,
}

pub fn encode_host_bridge_handshake(
    handshake: &WhatsAppHostBridgeHandshakeV1,
) -> Result<Vec<u8>, WhatsAppHostBridgeError> {
    validate_host_bridge_handshake(handshake)?;
    Ok(wire::WhatsAppHostBridgeHandshakeV1 {
        protocol_major: handshake.protocol_major,
        protocol_revision: handshake.protocol_revision,
        route_binding_sha256: handshake.route_binding_sha256.to_vec(),
    }
    .encode_to_vec())
}

pub fn decode_host_bridge_handshake(
    bytes: &[u8],
) -> Result<WhatsAppHostBridgeHandshakeV1, WhatsAppHostBridgeError> {
    let payload = wire::WhatsAppHostBridgeHandshakeV1::decode(bytes)
        .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?;
    let handshake = WhatsAppHostBridgeHandshakeV1 {
        protocol_major: payload.protocol_major,
        protocol_revision: payload.protocol_revision,
        route_binding_sha256: payload
            .route_binding_sha256
            .try_into()
            .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?,
    };
    validate_host_bridge_handshake(&handshake)?;
    Ok(handshake)
}

pub fn encode_host_bridge_handshake_accepted() -> Vec<u8> {
    wire::WhatsAppHostBridgeHandshakeAcceptedV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
    }
    .encode_to_vec()
}

pub fn decode_host_bridge_handshake_accepted(bytes: &[u8]) -> Result<(), WhatsAppHostBridgeError> {
    let accepted = wire::WhatsAppHostBridgeHandshakeAcceptedV1::decode(bytes)
        .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?;
    (accepted.protocol_major == HOST_BRIDGE_PROTOCOL_MAJOR
        && accepted.protocol_revision == HOST_BRIDGE_PROTOCOL_REVISION)
        .then_some(())
        .ok_or(WhatsAppHostBridgeError::InvalidProtocol)
}

pub fn validate_host_bridge_handshake(
    handshake: &WhatsAppHostBridgeHandshakeV1,
) -> Result<(), WhatsAppHostBridgeError> {
    if handshake.protocol_major != HOST_BRIDGE_PROTOCOL_MAJOR
        || handshake.protocol_revision != HOST_BRIDGE_PROTOCOL_REVISION
    {
        return Err(WhatsAppHostBridgeError::InvalidProtocol);
    }
    Ok(())
}

pub fn validate_host_bridge_envelope(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<(), WhatsAppHostBridgeError> {
    if envelope.protocol_major != HOST_BRIDGE_PROTOCOL_MAJOR
        || envelope.protocol_revision != HOST_BRIDGE_PROTOCOL_REVISION
    {
        return Err(WhatsAppHostBridgeError::InvalidProtocol);
    }
    for value in [
        envelope.account_id.as_str(),
        envelope.provider_event_id.as_str(),
    ] {
        if value.trim().is_empty() {
            return Err(WhatsAppHostBridgeError::EmptyField);
        }
    }
    if envelope.observed_at_unix_seconds <= 0 {
        return Err(WhatsAppHostBridgeError::InvalidTimestamp);
    }
    let operational_account_id = match &envelope.observation {
        WhatsAppHostObservationV1::OperationalMessage(value) => Some(value.account_id.as_str()),
        WhatsAppHostObservationV1::OperationalDialog(value) => Some(value.account_id.as_str()),
        WhatsAppHostObservationV1::OperationalParticipant(value) => Some(value.account_id.as_str()),
        _ => None,
    };
    if operational_account_id.is_some_and(|account_id| account_id != envelope.account_id) {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    let embedded_observed_at = match &envelope.observation {
        WhatsAppHostObservationV1::OperationalDialog(value) => Some(value.observed_at_unix_seconds),
        WhatsAppHostObservationV1::OperationalParticipant(value) => {
            Some(value.observed_at_unix_seconds)
        }
        _ => None,
    };
    if embedded_observed_at
        .is_some_and(|observed_at| observed_at != envelope.observed_at_unix_seconds)
    {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    validate_observation(&envelope.observation)
}

pub fn validate_host_command_claim(
    claim: &WhatsAppHostCommandClaimV1,
) -> Result<(), WhatsAppHostBridgeError> {
    if claim.account_id.trim().is_empty() || claim.host_claim_id.trim().is_empty() {
        return Err(WhatsAppHostBridgeError::EmptyField);
    }
    if claim.account_id.len() > crate::MAX_ID_LEN
        || claim.host_claim_id.len() > crate::MAX_ID_LEN
        || claim.lease_seconds == 0
        || claim.lease_seconds > 3600
        || claim.limit == 0
        || claim.limit > 500
    {
        return Err(WhatsAppHostBridgeError::InvalidProtocol);
    }
    Ok(())
}

pub fn encode_host_bridge_payload(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<Vec<u8>, WhatsAppHostBridgeError> {
    validate_host_bridge_envelope(envelope)?;
    let observation = wire::WhatsAppHostObservationV1 {
        protocol_major: envelope.protocol_major,
        protocol_revision: envelope.protocol_revision,
        account_id: envelope.account_id.clone(),
        provider_event_id: envelope.provider_event_id.clone(),
        observed_at_unix_seconds: envelope.observed_at_unix_seconds,
        observation: Some(observation_to_wire(&envelope.observation)),
    };
    Ok(wire::WhatsAppHostBridgeOperationV1 {
        operation: Some(
            wire::whats_app_host_bridge_operation_v1::Operation::Observation(Box::new(observation)),
        ),
    }
    .encode_to_vec())
}

pub fn encode_host_command_claim(
    claim: &WhatsAppHostCommandClaimV1,
) -> Result<Vec<u8>, WhatsAppHostBridgeError> {
    validate_host_command_claim(claim)?;
    Ok(wire::WhatsAppHostBridgeOperationV1 {
        operation: Some(
            wire::whats_app_host_bridge_operation_v1::Operation::ClaimCommands(
                wire::WhatsAppHostCommandClaimV1 {
                    account_id: claim.account_id.clone(),
                    host_claim_id: claim.host_claim_id.clone(),
                    lease_seconds: claim.lease_seconds,
                    limit: claim.limit,
                },
            ),
        ),
    }
    .encode_to_vec())
}

pub fn decode_host_bridge_operation(
    bytes: &[u8],
) -> Result<WhatsAppHostBridgeOperationV1, WhatsAppHostBridgeError> {
    use wire::whats_app_host_bridge_operation_v1::Operation;

    let operation = wire::WhatsAppHostBridgeOperationV1::decode(bytes)
        .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?
        .operation
        .ok_or(WhatsAppHostBridgeError::InvalidProtocol)?;
    match operation {
        Operation::Observation(payload) => {
            let payload = *payload;
            let observation = payload
                .observation
                .ok_or(WhatsAppHostBridgeError::InvalidProtocol)
                .and_then(observation_from_wire)?;
            let envelope = WhatsAppHostBridgeEnvelopeV1 {
                protocol_major: payload.protocol_major,
                protocol_revision: payload.protocol_revision,
                account_id: payload.account_id,
                provider_event_id: payload.provider_event_id,
                observed_at_unix_seconds: payload.observed_at_unix_seconds,
                observation,
            };
            validate_host_bridge_envelope(&envelope)?;
            Ok(WhatsAppHostBridgeOperationV1::Observation(Box::new(
                envelope,
            )))
        }
        Operation::ClaimCommands(payload) => {
            let claim = WhatsAppHostCommandClaimV1 {
                account_id: payload.account_id,
                host_claim_id: payload.host_claim_id,
                lease_seconds: payload.lease_seconds,
                limit: payload.limit,
            };
            validate_host_command_claim(&claim)?;
            Ok(WhatsAppHostBridgeOperationV1::ClaimCommands(claim))
        }
    }
}

pub fn decode_host_bridge_payload(
    bytes: &[u8],
) -> Result<WhatsAppHostBridgeEnvelopeV1, WhatsAppHostBridgeError> {
    match decode_host_bridge_operation(bytes)? {
        WhatsAppHostBridgeOperationV1::Observation(envelope) => Ok(*envelope),
        WhatsAppHostBridgeOperationV1::ClaimCommands(_) => {
            Err(WhatsAppHostBridgeError::InvalidProtocol)
        }
    }
}

pub fn encode_host_bridge_observation_accepted(
    provider_event_id: &str,
) -> Result<Vec<u8>, WhatsAppHostBridgeError> {
    if provider_event_id.trim().is_empty() || provider_event_id.len() > crate::MAX_ID_LEN {
        return Err(WhatsAppHostBridgeError::EmptyField);
    }
    Ok(wire::WhatsAppHostBridgeResponseV1 {
        response: Some(
            wire::whats_app_host_bridge_response_v1::Response::ObservationAccepted(
                wire::WhatsAppObservationAcceptedV1 {
                    provider_event_id: provider_event_id.to_owned(),
                },
            ),
        ),
    }
    .encode_to_vec())
}

pub fn decode_host_bridge_observation_accepted(
    bytes: &[u8],
) -> Result<String, WhatsAppHostBridgeError> {
    use wire::whats_app_host_bridge_response_v1::Response;

    match wire::WhatsAppHostBridgeResponseV1::decode(bytes)
        .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?
        .response
        .ok_or(WhatsAppHostBridgeError::InvalidProtocol)?
    {
        Response::ObservationAccepted(value)
            if !value.provider_event_id.trim().is_empty()
                && value.provider_event_id.len() <= crate::MAX_ID_LEN =>
        {
            Ok(value.provider_event_id)
        }
        Response::ObservationAccepted(_) | Response::CommandLease(_) => {
            Err(WhatsAppHostBridgeError::InvalidProtocol)
        }
    }
}

pub fn encode_host_bridge_command_lease(commands: &[WhatsAppProviderCommand]) -> Vec<u8> {
    wire::WhatsAppHostBridgeResponseV1 {
        response: Some(
            wire::whats_app_host_bridge_response_v1::Response::CommandLease(
                wire::WhatsAppHostCommandLeaseV1 {
                    command: commands.iter().map(client_wire::command_message).collect(),
                },
            ),
        ),
    }
    .encode_to_vec()
}

pub fn decode_host_bridge_command_lease(
    bytes: &[u8],
) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppHostBridgeError> {
    use wire::whats_app_host_bridge_response_v1::Response;

    match wire::WhatsAppHostBridgeResponseV1::decode(bytes)
        .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)?
        .response
        .ok_or(WhatsAppHostBridgeError::InvalidProtocol)?
    {
        Response::CommandLease(value) => value
            .command
            .into_iter()
            .map(|command| {
                client_wire::decode_command(&command.encode_to_vec())
                    .map_err(|_| WhatsAppHostBridgeError::InvalidProtocol)
            })
            .collect(),
        Response::ObservationAccepted(_) => Err(WhatsAppHostBridgeError::InvalidProtocol),
    }
}

fn observation_to_wire(
    observation: &WhatsAppHostObservationV1,
) -> wire::whats_app_host_observation_v1::Observation {
    use wire::whats_app_host_observation_v1::Observation;
    match observation {
        WhatsAppHostObservationV1::RuntimeState { state } => {
            Observation::RuntimeState(wire::RuntimeState {
                state: state.clone(),
            })
        }
        WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id,
            provider_message_id,
            sender_id,
        } => Observation::MessageIdentity(wire::MessageIdentity {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            sender_id: sender_id.clone(),
        }),
        WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id,
            provider_message_id,
        } => Observation::MessageUpdated(wire::MessageUpdated {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id,
            provider_message_id,
        } => Observation::MessageDeleted(wire::MessageDeleted {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        WhatsAppHostObservationV1::Receipt {
            provider_chat_id,
            provider_message_id,
            delivery_state,
        } => Observation::Receipt(wire::Receipt {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            delivery_state: delivery_state.clone(),
        }),
        WhatsAppHostObservationV1::Reaction {
            provider_chat_id,
            provider_message_id,
            actor_id,
            emoji,
            is_active,
        } => Observation::Reaction(wire::Reaction {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            actor_id: actor_id.clone(),
            emoji: emoji.clone(),
            is_active: *is_active,
        }),
        WhatsAppHostObservationV1::Dialog {
            provider_chat_id,
            title,
            kind,
        } => Observation::Dialog(wire::Dialog {
            provider_chat_id: provider_chat_id.clone(),
            title: title.clone(),
            kind: kind.clone(),
        }),
        WhatsAppHostObservationV1::Participant {
            provider_chat_id,
            provider_identity_id,
            display_name,
        } => Observation::Participant(wire::Participant {
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
            display_name: display_name.clone(),
        }),
        WhatsAppHostObservationV1::Presence {
            provider_chat_id,
            provider_identity_id,
            state,
        } => Observation::Presence(wire::Presence {
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
            state: state.clone(),
        }),
        WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id,
            provider_message_id,
            provider_media_id,
            media_kind,
            filename,
            content_type,
            declared_size,
        } => Observation::MediaMetadata(wire::MediaMetadata {
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            provider_media_id: provider_media_id.clone(),
            media_kind: media_kind.clone(),
            filename: filename.clone(),
            content_type: content_type.clone(),
            declared_size: *declared_size,
        }),
        WhatsAppHostObservationV1::CallMetadata {
            provider_call_id,
            provider_chat_id,
            direction,
            state,
        } => Observation::CallMetadata(wire::CallMetadata {
            provider_call_id: provider_call_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            direction: direction.clone(),
            state: state.clone(),
        }),
        WhatsAppHostObservationV1::StatusMetadata {
            provider_status_id,
            sender_id,
        } => Observation::StatusMetadata(wire::StatusMetadata {
            provider_status_id: provider_status_id.clone(),
            sender_id: sender_id.clone(),
        }),
        WhatsAppHostObservationV1::StatusViewMetadata {
            provider_status_id,
            viewer_id,
        } => Observation::StatusViewMetadata(wire::StatusViewMetadata {
            provider_status_id: provider_status_id.clone(),
            viewer_id: viewer_id.clone(),
        }),
        WhatsAppHostObservationV1::StatusDeletedMetadata { provider_status_id } => {
            Observation::StatusDeletedMetadata(wire::StatusDeletedMetadata {
                provider_status_id: provider_status_id.clone(),
            })
        }
        WhatsAppHostObservationV1::SessionLinked {
            secret_ref,
            revision,
        } => Observation::SessionLinked(wire::SessionLinkedMetadata {
            secret_ref: secret_ref.clone(),
            revision: *revision,
        }),
        WhatsAppHostObservationV1::SessionRevoked => {
            Observation::SessionRevoked(wire::SessionRevokedMetadata {})
        }
        WhatsAppHostObservationV1::CommandResult {
            operation_id,
            provider_request_id,
            succeeded,
            host_claim_id,
        } => Observation::CommandResult(wire::CommandResultMetadata {
            operation_id: operation_id.clone(),
            provider_request_id: provider_request_id.clone(),
            succeeded: *succeeded,
            host_claim_id: host_claim_id.clone(),
        }),
        WhatsAppHostObservationV1::OperationalMessage(value) => {
            Observation::OperationalMessage(client_wire::message_to_wire(value))
        }
        WhatsAppHostObservationV1::OperationalDialog(value) => {
            Observation::OperationalDialog(client_wire::dialog_to_wire(value))
        }
        WhatsAppHostObservationV1::OperationalParticipant(value) => {
            Observation::OperationalParticipant(client_wire::participant_to_wire(value))
        }
        WhatsAppHostObservationV1::OperationalParticipantRemoved {
            provider_chat_id,
            provider_identity_id,
        } => Observation::OperationalParticipantRemoved(wire::OperationalParticipantRemoved {
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
        }),
        WhatsAppHostObservationV1::OperationalResyncState { complete } => {
            Observation::OperationalResyncState(wire::OperationalResyncState {
                complete: *complete,
            })
        }
    }
}

fn observation_from_wire(
    observation: wire::whats_app_host_observation_v1::Observation,
) -> Result<WhatsAppHostObservationV1, WhatsAppHostBridgeError> {
    use wire::whats_app_host_observation_v1::Observation;
    Ok(match observation {
        Observation::RuntimeState(value) => {
            WhatsAppHostObservationV1::RuntimeState { state: value.state }
        }
        Observation::MessageIdentity(value) => WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            sender_id: value.sender_id,
        },
        Observation::MessageUpdated(value) => WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
        },
        Observation::MessageDeleted(value) => WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
        },
        Observation::Receipt(value) => WhatsAppHostObservationV1::Receipt {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            delivery_state: value.delivery_state,
        },
        Observation::Reaction(value) => WhatsAppHostObservationV1::Reaction {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            actor_id: value.actor_id,
            emoji: value.emoji,
            is_active: value.is_active,
        },
        Observation::Dialog(value) => WhatsAppHostObservationV1::Dialog {
            provider_chat_id: value.provider_chat_id,
            title: value.title,
            kind: value.kind,
        },
        Observation::Participant(value) => WhatsAppHostObservationV1::Participant {
            provider_chat_id: value.provider_chat_id,
            provider_identity_id: value.provider_identity_id,
            display_name: value.display_name,
        },
        Observation::Presence(value) => WhatsAppHostObservationV1::Presence {
            provider_chat_id: value.provider_chat_id,
            provider_identity_id: value.provider_identity_id,
            state: value.state,
        },
        Observation::MediaMetadata(value) => WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            provider_media_id: value.provider_media_id,
            media_kind: value.media_kind,
            filename: value.filename,
            content_type: value.content_type,
            declared_size: value.declared_size,
        },
        Observation::CallMetadata(value) => WhatsAppHostObservationV1::CallMetadata {
            provider_call_id: value.provider_call_id,
            provider_chat_id: value.provider_chat_id,
            direction: value.direction,
            state: value.state,
        },
        Observation::StatusMetadata(value) => WhatsAppHostObservationV1::StatusMetadata {
            provider_status_id: value.provider_status_id,
            sender_id: value.sender_id,
        },
        Observation::StatusViewMetadata(value) => WhatsAppHostObservationV1::StatusViewMetadata {
            provider_status_id: value.provider_status_id,
            viewer_id: value.viewer_id,
        },
        Observation::StatusDeletedMetadata(value) => {
            WhatsAppHostObservationV1::StatusDeletedMetadata {
                provider_status_id: value.provider_status_id,
            }
        }
        Observation::SessionLinked(value) => WhatsAppHostObservationV1::SessionLinked {
            secret_ref: value.secret_ref,
            revision: value.revision,
        },
        Observation::SessionRevoked(_) => WhatsAppHostObservationV1::SessionRevoked,
        Observation::CommandResult(value) => WhatsAppHostObservationV1::CommandResult {
            operation_id: value.operation_id,
            provider_request_id: value.provider_request_id,
            succeeded: value.succeeded,
            host_claim_id: value.host_claim_id,
        },
        Observation::OperationalMessage(value) => {
            WhatsAppHostObservationV1::OperationalMessage(client_wire::parse_message(value))
        }
        Observation::OperationalDialog(value) => {
            WhatsAppHostObservationV1::OperationalDialog(client_wire::parse_dialog(value))
        }
        Observation::OperationalParticipant(value) => {
            WhatsAppHostObservationV1::OperationalParticipant(client_wire::parse_participant(value))
        }
        Observation::OperationalParticipantRemoved(value) => {
            WhatsAppHostObservationV1::OperationalParticipantRemoved {
                provider_chat_id: value.provider_chat_id,
                provider_identity_id: value.provider_identity_id,
            }
        }
        Observation::OperationalResyncState(value) => {
            WhatsAppHostObservationV1::OperationalResyncState {
                complete: value.complete,
            }
        }
    })
}

fn validate_observation(
    observation: &WhatsAppHostObservationV1,
) -> Result<(), WhatsAppHostBridgeError> {
    let fields = match observation {
        WhatsAppHostObservationV1::RuntimeState { state } => vec![state.as_str()],
        WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id,
            provider_message_id,
            sender_id,
        } => vec![
            provider_chat_id.as_str(),
            provider_message_id.as_str(),
            sender_id.as_str(),
        ],
        WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id,
            provider_message_id,
        }
        | WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id,
            provider_message_id,
        } => vec![provider_chat_id.as_str(), provider_message_id.as_str()],
        WhatsAppHostObservationV1::Receipt {
            provider_chat_id,
            provider_message_id,
            delivery_state,
        } => vec![
            provider_chat_id.as_str(),
            provider_message_id.as_str(),
            delivery_state.as_str(),
        ],
        WhatsAppHostObservationV1::Reaction {
            provider_chat_id,
            provider_message_id,
            actor_id,
            ..
        } => vec![
            provider_chat_id.as_str(),
            provider_message_id.as_str(),
            actor_id.as_str(),
        ],
        WhatsAppHostObservationV1::Dialog {
            provider_chat_id,
            title,
            kind,
        } => vec![provider_chat_id.as_str(), title.as_str(), kind.as_str()],
        WhatsAppHostObservationV1::Participant {
            provider_chat_id,
            provider_identity_id,
            display_name,
        } => vec![
            provider_chat_id.as_str(),
            provider_identity_id.as_str(),
            display_name.as_str(),
        ],
        WhatsAppHostObservationV1::Presence {
            provider_chat_id,
            provider_identity_id,
            state,
        } => vec![
            provider_chat_id.as_str(),
            provider_identity_id.as_str(),
            state.as_str(),
        ],
        WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id,
            provider_message_id,
            provider_media_id,
            media_kind,
            ..
        } => vec![
            provider_chat_id.as_str(),
            provider_message_id.as_str(),
            provider_media_id.as_str(),
            media_kind.as_str(),
        ],
        WhatsAppHostObservationV1::CallMetadata {
            provider_call_id,
            provider_chat_id,
            direction,
            state,
        } => vec![
            provider_call_id.as_str(),
            provider_chat_id.as_str(),
            direction.as_str(),
            state.as_str(),
        ],
        WhatsAppHostObservationV1::StatusMetadata {
            provider_status_id,
            sender_id,
        } => vec![provider_status_id.as_str(), sender_id.as_str()],
        WhatsAppHostObservationV1::StatusViewMetadata {
            provider_status_id,
            viewer_id,
        } => vec![provider_status_id.as_str(), viewer_id.as_str()],
        WhatsAppHostObservationV1::StatusDeletedMetadata { provider_status_id } => {
            vec![provider_status_id.as_str()]
        }
        WhatsAppHostObservationV1::SessionLinked { secret_ref, .. } => vec![secret_ref.as_str()],
        WhatsAppHostObservationV1::SessionRevoked => Vec::new(),
        WhatsAppHostObservationV1::CommandResult {
            operation_id,
            host_claim_id,
            ..
        } => {
            vec![operation_id.as_str(), host_claim_id.as_str()]
        }
        WhatsAppHostObservationV1::OperationalMessage(value) => vec![
            value.account_id.as_str(),
            value.provider_chat_id.as_str(),
            value.provider_message_id.as_str(),
            value.sender_id.as_str(),
        ],
        WhatsAppHostObservationV1::OperationalDialog(value) => vec![
            value.account_id.as_str(),
            value.provider_chat_id.as_str(),
            value.title.as_str(),
            value.kind.as_str(),
        ],
        WhatsAppHostObservationV1::OperationalParticipant(value) => vec![
            value.account_id.as_str(),
            value.provider_chat_id.as_str(),
            value.provider_identity_id.as_str(),
            value.display_name.as_str(),
            value.role.as_str(),
            value.status.as_str(),
        ],
        WhatsAppHostObservationV1::OperationalParticipantRemoved {
            provider_chat_id,
            provider_identity_id,
        } => vec![provider_chat_id.as_str(), provider_identity_id.as_str()],
        WhatsAppHostObservationV1::OperationalResyncState { .. } => Vec::new(),
    };
    if fields.iter().any(|value| value.trim().is_empty()) {
        return Err(WhatsAppHostBridgeError::EmptyField);
    }
    if let WhatsAppHostObservationV1::SessionLinked { revision, .. } = observation
        && *revision == 0
    {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    if let WhatsAppHostObservationV1::MediaMetadata {
        filename,
        content_type,
        ..
    } = observation
        && (filename.as_deref().is_some_and(str::is_empty)
            || content_type.as_deref().is_some_and(str::is_empty))
    {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    if let WhatsAppHostObservationV1::OperationalMessage(value) = observation
        && value
            .delivery_state
            .as_deref()
            .is_some_and(|state| state.trim().is_empty())
    {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    let operational_event = match observation {
        WhatsAppHostObservationV1::OperationalMessage(value) => {
            Some(WhatsAppProviderEvent::MessageObserved(value.clone()))
        }
        WhatsAppHostObservationV1::OperationalDialog(value) => {
            Some(WhatsAppProviderEvent::DialogObserved(value.clone()))
        }
        WhatsAppHostObservationV1::OperationalParticipant(value) => {
            Some(WhatsAppProviderEvent::ParticipantObserved(value.clone()))
        }
        _ => None,
    };
    if operational_event
        .as_ref()
        .is_some_and(|event| validate_event(event).is_err())
    {
        return Err(WhatsAppHostBridgeError::ForbiddenContent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_round_trips_only_the_exact_protocol_and_route_binding() {
        let handshake = WhatsAppHostBridgeHandshakeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            route_binding_sha256: [7; 32],
        };

        let encoded = encode_host_bridge_handshake(&handshake).expect("encoded handshake");

        assert_eq!(decode_host_bridge_handshake(&encoded), Ok(handshake));
    }

    #[test]
    fn host_operations_keep_observation_and_command_claim_separate() {
        let observation = WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::RuntimeState {
                state: "running".to_owned(),
            },
        };
        let observation_bytes =
            encode_host_bridge_payload(&observation).expect("encoded observation");
        assert_eq!(
            decode_host_bridge_operation(&observation_bytes),
            Ok(WhatsAppHostBridgeOperationV1::Observation(Box::new(
                observation,
            )))
        );

        let claim = WhatsAppHostCommandClaimV1 {
            account_id: "wa-1".to_owned(),
            host_claim_id: "host-1".to_owned(),
            lease_seconds: 30,
            limit: 4,
        };
        let claim_bytes = encode_host_command_claim(&claim).expect("encoded claim");
        assert_eq!(
            decode_host_bridge_operation(&claim_bytes),
            Ok(WhatsAppHostBridgeOperationV1::ClaimCommands(claim))
        );
        assert_eq!(
            decode_host_bridge_payload(&claim_bytes),
            Err(WhatsAppHostBridgeError::InvalidProtocol)
        );
    }

    #[test]
    fn host_responses_reject_the_other_response_kind() {
        let accepted =
            encode_host_bridge_observation_accepted("event-1").expect("accepted observation");
        assert_eq!(
            decode_host_bridge_observation_accepted(&accepted),
            Ok("event-1".to_owned())
        );
        assert_eq!(
            decode_host_bridge_command_lease(&accepted),
            Err(WhatsAppHostBridgeError::InvalidProtocol)
        );

        let command = WhatsAppProviderCommand::SendText {
            operation_id: "op-1".to_owned(),
            account_id: "wa-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            text: "hello".to_owned(),
        };
        let lease = encode_host_bridge_command_lease(std::slice::from_ref(&command));
        assert_eq!(decode_host_bridge_command_lease(&lease), Ok(vec![command]));
        assert_eq!(
            decode_host_bridge_observation_accepted(&lease),
            Err(WhatsAppHostBridgeError::InvalidProtocol)
        );
    }
}
