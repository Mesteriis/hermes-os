//! Protobuf mapping for the exact WhatsApp operational replay capability.

use prost::Message;

use crate::{
    client_wire::{ClientWireError, event_to_wire, parse_event},
    realtime::{
        WhatsAppOperationalReplayFrameV1, WhatsAppOperationalReplayRequestV1,
        WhatsAppOperationalReplayResponseV1, validate_operational_replay_request,
        validate_operational_replay_response,
    },
    realtime_wire_generated as wire,
};

pub fn encode_operational_replay_request(
    request: &WhatsAppOperationalReplayRequestV1,
) -> Result<Vec<u8>, ClientWireError> {
    validate_operational_replay_request(request).map_err(|_| ClientWireError::InvalidPayload)?;
    Ok(wire::WhatsAppOperationalReplayRequestV1 {
        account_id: request.account_id.clone(),
        after_sequence: request.after_sequence,
        limit: request.limit,
    }
    .encode_to_vec())
}

pub fn decode_operational_replay_request(
    bytes: &[u8],
) -> Result<WhatsAppOperationalReplayRequestV1, ClientWireError> {
    let wire = wire::WhatsAppOperationalReplayRequestV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?;
    let request = WhatsAppOperationalReplayRequestV1 {
        account_id: wire.account_id,
        after_sequence: wire.after_sequence,
        limit: wire.limit,
    };
    validate_operational_replay_request(&request).map_err(|_| ClientWireError::InvalidPayload)?;
    Ok(request)
}

pub fn encode_operational_replay_response(
    response: &WhatsAppOperationalReplayResponseV1,
) -> Result<Vec<u8>, ClientWireError> {
    validate_operational_replay_response(response).map_err(|_| ClientWireError::InvalidPayload)?;
    Ok(wire::WhatsAppOperationalReplayResponseV1 {
        earliest_available_sequence: response.earliest_available_sequence,
        latest_available_sequence: response.latest_available_sequence,
        frame: response
            .frames
            .iter()
            .map(|frame| wire::WhatsAppOperationalReplayFrameV1 {
                sequence: frame.sequence,
                event: Some(event_to_wire(&frame.event)),
            })
            .collect(),
        next_sequence: response.next_sequence,
        reset_required: response.reset_required,
        account_id: response.account_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_operational_replay_response(
    bytes: &[u8],
) -> Result<WhatsAppOperationalReplayResponseV1, ClientWireError> {
    let wire = wire::WhatsAppOperationalReplayResponseV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?;
    let response = WhatsAppOperationalReplayResponseV1 {
        account_id: wire.account_id,
        earliest_available_sequence: wire.earliest_available_sequence,
        latest_available_sequence: wire.latest_available_sequence,
        frames: wire
            .frame
            .into_iter()
            .map(|frame| {
                Ok(WhatsAppOperationalReplayFrameV1 {
                    sequence: frame.sequence,
                    event: parse_event(frame.event.ok_or(ClientWireError::MissingVariant)?)?,
                })
            })
            .collect::<Result<Vec<_>, ClientWireError>>()?,
        next_sequence: wire.next_sequence,
        reset_required: wire.reset_required,
    };
    validate_operational_replay_response(&response).map_err(|_| ClientWireError::InvalidPayload)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::{WhatsAppProviderEvent, WhatsAppRuntimeState};

    use super::*;

    #[test]
    fn replay_round_trips_with_exact_account_scope() {
        let request = WhatsAppOperationalReplayRequestV1 {
            account_id: "wa-1".into(),
            after_sequence: 7,
            limit: 20,
        };
        assert_eq!(
            decode_operational_replay_request(
                &encode_operational_replay_request(&request).expect("encode request")
            ),
            Ok(request)
        );

        let response = WhatsAppOperationalReplayResponseV1 {
            account_id: "wa-1".into(),
            earliest_available_sequence: Some(8),
            latest_available_sequence: Some(8),
            frames: vec![WhatsAppOperationalReplayFrameV1 {
                sequence: 8,
                event: WhatsAppProviderEvent::RuntimeStateChanged {
                    account_id: "wa-1".into(),
                    state: WhatsAppRuntimeState::Running,
                    observed_at_unix_seconds: 1_782_504_000,
                },
            }],
            next_sequence: 8,
            reset_required: false,
        };
        let encoded = encode_operational_replay_response(&response).expect("encode response");
        assert_eq!(decode_operational_replay_response(&encoded), Ok(response));
    }
}
