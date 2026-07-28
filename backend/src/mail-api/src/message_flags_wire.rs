use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    message_flags::{
        MailMessageFlagAcceptedV1, MailMessageFlagCommandV1, MailMessageFlagKindV1,
        MailMessageFlagOperationOutcomeV1, MailMessageFlagOperationStatusV1,
        MailMessageFlagStatusRequestV1, validate_message_flag_accepted,
        validate_message_flag_command, validate_message_flag_status,
        validate_message_flag_status_request,
    },
    message_flags_wire_generated as wire,
};

pub fn encode_message_flag_command(
    command: &MailMessageFlagCommandV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_flag_command(command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageFlagCommandV1 {
        operation_id: command.operation_id.clone(),
        connection_id: command.connection_id.clone(),
        message_id: command.message_id.clone(),
        kind: flag_kind_to_wire(command.kind),
        target_value: command.target_value,
    }
    .encode_to_vec())
}

pub fn decode_message_flag_command(
    bytes: &[u8],
) -> Result<MailMessageFlagCommandV1, MailClientWireErrorV1> {
    let value = wire::MailMessageFlagCommandV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let command = MailMessageFlagCommandV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        kind: flag_kind_from_wire(value.kind)?,
        target_value: value.target_value,
    };
    validate_message_flag_command(&command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_flag_command(&command)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(command)
}

pub fn encode_message_flag_accepted(
    accepted: &MailMessageFlagAcceptedV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_flag_accepted(accepted).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageFlagAcceptedV1 {
        operation_id: accepted.operation_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_flag_accepted(
    bytes: &[u8],
) -> Result<MailMessageFlagAcceptedV1, MailClientWireErrorV1> {
    let value = wire::MailMessageFlagAcceptedV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let accepted = MailMessageFlagAcceptedV1 {
        operation_id: value.operation_id,
    };
    validate_message_flag_accepted(&accepted).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_flag_accepted(&accepted)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(accepted)
}

pub fn encode_message_flag_status_request(
    request: &MailMessageFlagStatusRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_flag_status_request(request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageFlagStatusRequestV1 {
        operation_id: request.operation_id.clone(),
        connection_id: request.connection_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_flag_status_request(
    bytes: &[u8],
) -> Result<MailMessageFlagStatusRequestV1, MailClientWireErrorV1> {
    let value = wire::MailMessageFlagStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailMessageFlagStatusRequestV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
    };
    validate_message_flag_status_request(&request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_flag_status_request(&request)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

pub fn encode_message_flag_status_response(
    status: Option<&MailMessageFlagOperationStatusV1>,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    let status = status.map(status_to_wire).transpose()?;
    Ok(wire::MailMessageFlagStatusResponseV1 { status }.encode_to_vec())
}

pub fn decode_message_flag_status_response(
    bytes: &[u8],
) -> Result<Option<MailMessageFlagOperationStatusV1>, MailClientWireErrorV1> {
    let status = wire::MailMessageFlagStatusResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .status
        .map(status_from_wire)
        .transpose()?;
    if encode_message_flag_status_response(status.as_ref())? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(status)
}

fn status_to_wire(
    status: &MailMessageFlagOperationStatusV1,
) -> Result<wire::MailMessageFlagOperationStatusV1, MailClientWireErrorV1> {
    validate_message_flag_status(status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageFlagOperationStatusV1 {
        operation_id: status.operation_id.clone(),
        connection_id: status.connection_id.clone(),
        message_id: status.message_id.clone(),
        kind: flag_kind_to_wire(status.kind),
        target_value: status.target_value,
        outcome: outcome_to_wire(status.outcome),
        requested_at_unix_seconds: status.requested_at_unix_seconds,
        completed_at_unix_seconds: status.completed_at_unix_seconds,
        projection_revision: status.projection_revision,
    })
}

fn status_from_wire(
    value: wire::MailMessageFlagOperationStatusV1,
) -> Result<MailMessageFlagOperationStatusV1, MailClientWireErrorV1> {
    let status = MailMessageFlagOperationStatusV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        kind: flag_kind_from_wire(value.kind)?,
        target_value: value.target_value,
        outcome: outcome_from_wire(value.outcome)?,
        requested_at_unix_seconds: value.requested_at_unix_seconds,
        completed_at_unix_seconds: value.completed_at_unix_seconds,
        projection_revision: value.projection_revision,
    };
    validate_message_flag_status(&status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(status)
}

const fn flag_kind_to_wire(kind: MailMessageFlagKindV1) -> i32 {
    match kind {
        MailMessageFlagKindV1::Read => wire::MailMessageFlagKindV1::MailMessageFlagKindRead as i32,
        MailMessageFlagKindV1::Starred => {
            wire::MailMessageFlagKindV1::MailMessageFlagKindStarred as i32
        }
    }
}

fn flag_kind_from_wire(value: i32) -> Result<MailMessageFlagKindV1, MailClientWireErrorV1> {
    match wire::MailMessageFlagKindV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessageFlagKindV1::MailMessageFlagKindRead => Ok(MailMessageFlagKindV1::Read),
        wire::MailMessageFlagKindV1::MailMessageFlagKindStarred => {
            Ok(MailMessageFlagKindV1::Starred)
        }
        wire::MailMessageFlagKindV1::MailMessageFlagKindUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn outcome_to_wire(outcome: MailMessageFlagOperationOutcomeV1) -> i32 {
    match outcome {
        MailMessageFlagOperationOutcomeV1::Pending => {
            wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomePending as i32
        }
        MailMessageFlagOperationOutcomeV1::Succeeded => {
            wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeSucceeded as i32
        }
        MailMessageFlagOperationOutcomeV1::Rejected => {
            wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeRejected as i32
        }
        MailMessageFlagOperationOutcomeV1::OutcomeUnknown => {
            wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeUnknown as i32
        }
    }
}

fn outcome_from_wire(
    value: i32,
) -> Result<MailMessageFlagOperationOutcomeV1, MailClientWireErrorV1> {
    match wire::MailMessageFlagOperationOutcomeV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomePending => {
            Ok(MailMessageFlagOperationOutcomeV1::Pending)
        }
        wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeSucceeded => {
            Ok(MailMessageFlagOperationOutcomeV1::Succeeded)
        }
        wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeRejected => {
            Ok(MailMessageFlagOperationOutcomeV1::Rejected)
        }
        wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeUnknown => {
            Ok(MailMessageFlagOperationOutcomeV1::OutcomeUnknown)
        }
        wire::MailMessageFlagOperationOutcomeV1::MailMessageFlagOperationOutcomeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MailMessageFlagCommandV1 {
        MailMessageFlagCommandV1 {
            operation_id: "flag-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "provider-message-1".to_owned(),
            kind: MailMessageFlagKindV1::Read,
            target_value: true,
        }
    }

    #[test]
    fn command_and_status_round_trip_canonically() {
        let command = command();
        let encoded = encode_message_flag_command(&command).expect("encode command");
        assert_eq!(
            decode_message_flag_command(&encoded).expect("decode command"),
            command
        );

        let status = MailMessageFlagOperationStatusV1 {
            operation_id: command.operation_id,
            connection_id: command.connection_id,
            message_id: command.message_id,
            kind: command.kind,
            target_value: command.target_value,
            outcome: MailMessageFlagOperationOutcomeV1::Succeeded,
            requested_at_unix_seconds: 100,
            completed_at_unix_seconds: Some(101),
            projection_revision: Some(2),
        };
        let encoded =
            encode_message_flag_status_response(Some(&status)).expect("encode status response");
        assert_eq!(
            decode_message_flag_status_response(&encoded).expect("decode status response"),
            Some(status)
        );
    }

    #[test]
    fn decoder_rejects_unknown_enum_and_noncanonical_bytes() {
        let mut wire = wire::MailMessageFlagCommandV1 {
            operation_id: "flag-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "provider-message-1".to_owned(),
            kind: 999,
            target_value: true,
        }
        .encode_to_vec();
        assert_eq!(
            decode_message_flag_command(&wire),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
        wire = encode_message_flag_command(&command()).expect("encode command");
        wire.extend_from_slice(&[0x98, 0x06, 0x01]);
        assert_eq!(
            decode_message_flag_command(&wire),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
