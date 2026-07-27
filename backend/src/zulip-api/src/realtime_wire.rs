//! Protobuf mapping for public Zulip operational replay.

use prost::Message;

use crate::{
    client_wire::ZulipClientWireErrorV1,
    operational_wire::{decode_operational_event, encode_operational_event},
    realtime::{
        ZulipOperationalReplayFrameV1, ZulipOperationalReplayRequestV1,
        ZulipOperationalReplayResponseV1, validate_operational_replay_request,
        validate_operational_replay_response,
    },
    realtime_wire_generated as wire,
};

pub fn encode_operational_replay_request(
    request: &ZulipOperationalReplayRequestV1,
) -> Result<Vec<u8>, ZulipClientWireErrorV1> {
    validate_operational_replay_request(request)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(wire::ZulipOperationalReplayRequestV1 {
        account_id: request.account_id.clone(),
        after_sequence: request.after_sequence,
        limit: request.limit,
    }
    .encode_to_vec())
}

pub fn decode_operational_replay_request(
    bytes: &[u8],
) -> Result<ZulipOperationalReplayRequestV1, ZulipClientWireErrorV1> {
    let request = wire::ZulipOperationalReplayRequestV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    let request = ZulipOperationalReplayRequestV1 {
        account_id: request.account_id,
        after_sequence: request.after_sequence,
        limit: request.limit,
    };
    validate_operational_replay_request(&request)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(request)
}

pub fn encode_operational_replay_response(
    response: &ZulipOperationalReplayResponseV1,
) -> Result<Vec<u8>, ZulipClientWireErrorV1> {
    validate_operational_replay_response(response)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(wire::ZulipOperationalReplayResponseV1 {
        earliest_available_sequence: response.earliest_available_sequence,
        latest_available_sequence: response.latest_available_sequence,
        frame: response
            .frames
            .iter()
            .map(|frame| wire::ZulipOperationalReplayFrameV1 {
                sequence: frame.sequence,
                event: crate::operational_wire_generated::ZulipOperationalEventV1::decode(
                    encode_operational_event(&frame.event).as_slice(),
                )
                .ok(),
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
) -> Result<ZulipOperationalReplayResponseV1, ZulipClientWireErrorV1> {
    let response = wire::ZulipOperationalReplayResponseV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    let response = ZulipOperationalReplayResponseV1 {
        earliest_available_sequence: response.earliest_available_sequence,
        latest_available_sequence: response.latest_available_sequence,
        frames: response
            .frame
            .into_iter()
            .map(|frame| {
                let event = frame.event.ok_or(ZulipClientWireErrorV1::MissingVariant)?;
                Ok(ZulipOperationalReplayFrameV1 {
                    sequence: frame.sequence,
                    event: decode_operational_event(&event.encode_to_vec())?,
                })
            })
            .collect::<Result<_, _>>()?,
        next_sequence: response.next_sequence,
        reset_required: response.reset_required,
        account_id: response.account_id,
    };
    validate_operational_replay_response(&response)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::operational::{ZulipOperationalEventKindV1, ZulipOperationalEventV1};

    use super::*;

    #[test]
    fn replay_round_trips_typed_frames_and_reset() {
        let response = ZulipOperationalReplayResponseV1 {
            earliest_available_sequence: Some(1),
            latest_available_sequence: Some(2),
            frames: vec![ZulipOperationalReplayFrameV1 {
                sequence: 2,
                event: ZulipOperationalEventV1 {
                    account_id: "account".into(),
                    provider_event_id: 10,
                    provider_message_id: "20".into(),
                    provider_conversation_id: Some("direct:2".into()),
                    actor_id: Some("3".into()),
                    kind: ZulipOperationalEventKindV1::MessageUpdated,
                    content: Some("edited".into()),
                    topic: None,
                    reaction: None,
                    observed_at_unix_seconds: 30,
                },
            }],
            next_sequence: 2,
            reset_required: false,
            account_id: "account".into(),
        };
        assert_eq!(
            decode_operational_replay_response(
                &encode_operational_replay_response(&response).expect("encode")
            ),
            Ok(response)
        );
    }
}
