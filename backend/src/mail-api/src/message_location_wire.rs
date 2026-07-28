use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    message_location::{
        MailMessageLocationAcceptedV1, MailMessageLocationCommandV1, MailMessageLocationKindV1,
        MailMessageLocationOperationOutcomeV1, MailMessageLocationOperationStatusV1,
        MailMessageLocationStatusRequestV1, validate_message_location_accepted,
        validate_message_location_command, validate_message_location_status,
        validate_message_location_status_request,
    },
    message_location_wire_generated as wire,
};

pub fn encode_message_location_command(
    command: &MailMessageLocationCommandV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_location_command(command)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageLocationCommandV1 {
        operation_id: command.operation_id.clone(),
        connection_id: command.connection_id.clone(),
        message_id: command.message_id.clone(),
        kind: kind_to_wire(command.kind),
        target_folder_id: command.target_folder_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_location_command(
    bytes: &[u8],
) -> Result<MailMessageLocationCommandV1, MailClientWireErrorV1> {
    let value = wire::MailMessageLocationCommandV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let command = MailMessageLocationCommandV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        kind: kind_from_wire(value.kind)?,
        target_folder_id: value.target_folder_id,
    };
    validate_message_location_command(&command)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_location_command(&command)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(command)
}

pub fn encode_message_location_accepted(
    accepted: &MailMessageLocationAcceptedV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_location_accepted(accepted)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageLocationAcceptedV1 {
        operation_id: accepted.operation_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_location_accepted(
    bytes: &[u8],
) -> Result<MailMessageLocationAcceptedV1, MailClientWireErrorV1> {
    let value = wire::MailMessageLocationAcceptedV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let accepted = MailMessageLocationAcceptedV1 {
        operation_id: value.operation_id,
    };
    validate_message_location_accepted(&accepted)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_location_accepted(&accepted)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(accepted)
}

pub fn encode_message_location_status_request(
    request: &MailMessageLocationStatusRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_location_status_request(request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageLocationStatusRequestV1 {
        operation_id: request.operation_id.clone(),
        connection_id: request.connection_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_location_status_request(
    bytes: &[u8],
) -> Result<MailMessageLocationStatusRequestV1, MailClientWireErrorV1> {
    let value = wire::MailMessageLocationStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailMessageLocationStatusRequestV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
    };
    validate_message_location_status_request(&request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_location_status_request(&request)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

pub fn encode_message_location_status_response(
    status: Option<&MailMessageLocationOperationStatusV1>,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    let status = status.map(status_to_wire).transpose()?;
    Ok(wire::MailMessageLocationStatusResponseV1 { status }.encode_to_vec())
}

pub fn decode_message_location_status_response(
    bytes: &[u8],
) -> Result<Option<MailMessageLocationOperationStatusV1>, MailClientWireErrorV1> {
    let status = wire::MailMessageLocationStatusResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .status
        .map(status_from_wire)
        .transpose()?;
    if encode_message_location_status_response(status.as_ref())? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(status)
}

fn status_to_wire(
    status: &MailMessageLocationOperationStatusV1,
) -> Result<wire::MailMessageLocationOperationStatusV1, MailClientWireErrorV1> {
    validate_message_location_status(status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessageLocationOperationStatusV1 {
        operation_id: status.operation_id.clone(),
        connection_id: status.connection_id.clone(),
        message_id: status.message_id.clone(),
        kind: kind_to_wire(status.kind),
        target_folder_id: status.target_folder_id.clone(),
        outcome: outcome_to_wire(status.outcome),
        requested_at_unix_seconds: status.requested_at_unix_seconds,
        completed_at_unix_seconds: status.completed_at_unix_seconds,
        projection_revision: status.projection_revision,
    })
}

fn status_from_wire(
    value: wire::MailMessageLocationOperationStatusV1,
) -> Result<MailMessageLocationOperationStatusV1, MailClientWireErrorV1> {
    let status = MailMessageLocationOperationStatusV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        kind: kind_from_wire(value.kind)?,
        target_folder_id: value.target_folder_id,
        outcome: outcome_from_wire(value.outcome)?,
        requested_at_unix_seconds: value.requested_at_unix_seconds,
        completed_at_unix_seconds: value.completed_at_unix_seconds,
        projection_revision: value.projection_revision,
    };
    validate_message_location_status(&status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(status)
}

const fn kind_to_wire(kind: MailMessageLocationKindV1) -> i32 {
    match kind {
        MailMessageLocationKindV1::Archive => {
            wire::MailMessageLocationKindV1::MailMessageLocationKindArchive as i32
        }
        MailMessageLocationKindV1::Trash => {
            wire::MailMessageLocationKindV1::MailMessageLocationKindTrash as i32
        }
        MailMessageLocationKindV1::Restore => {
            wire::MailMessageLocationKindV1::MailMessageLocationKindRestore as i32
        }
        MailMessageLocationKindV1::Move => {
            wire::MailMessageLocationKindV1::MailMessageLocationKindMove as i32
        }
    }
}

fn kind_from_wire(value: i32) -> Result<MailMessageLocationKindV1, MailClientWireErrorV1> {
    match wire::MailMessageLocationKindV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessageLocationKindV1::MailMessageLocationKindArchive => {
            Ok(MailMessageLocationKindV1::Archive)
        }
        wire::MailMessageLocationKindV1::MailMessageLocationKindTrash => {
            Ok(MailMessageLocationKindV1::Trash)
        }
        wire::MailMessageLocationKindV1::MailMessageLocationKindRestore => {
            Ok(MailMessageLocationKindV1::Restore)
        }
        wire::MailMessageLocationKindV1::MailMessageLocationKindMove => {
            Ok(MailMessageLocationKindV1::Move)
        }
        wire::MailMessageLocationKindV1::MailMessageLocationKindUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

const fn outcome_to_wire(outcome: MailMessageLocationOperationOutcomeV1) -> i32 {
    match outcome {
        MailMessageLocationOperationOutcomeV1::Pending => {
            wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomePending
                as i32
        }
        MailMessageLocationOperationOutcomeV1::Succeeded => {
            wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeSucceeded
                as i32
        }
        MailMessageLocationOperationOutcomeV1::Rejected => {
            wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeRejected
                as i32
        }
        MailMessageLocationOperationOutcomeV1::Unsupported => {
            wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeUnsupported
                as i32
        }
        MailMessageLocationOperationOutcomeV1::OutcomeUnknown => {
            wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeUnknown
                as i32
        }
    }
}

fn outcome_from_wire(
    value: i32,
) -> Result<MailMessageLocationOperationOutcomeV1, MailClientWireErrorV1> {
    match wire::MailMessageLocationOperationOutcomeV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomePending => {
            Ok(MailMessageLocationOperationOutcomeV1::Pending)
        }
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeSucceeded => {
            Ok(MailMessageLocationOperationOutcomeV1::Succeeded)
        }
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeRejected => {
            Ok(MailMessageLocationOperationOutcomeV1::Rejected)
        }
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeUnsupported => {
            Ok(MailMessageLocationOperationOutcomeV1::Unsupported)
        }
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeUnknown => {
            Ok(MailMessageLocationOperationOutcomeV1::OutcomeUnknown)
        }
        wire::MailMessageLocationOperationOutcomeV1::MailMessageLocationOperationOutcomeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MailMessageLocationCommandV1 {
        MailMessageLocationCommandV1 {
            operation_id: "location-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            kind: MailMessageLocationKindV1::Move,
            target_folder_id: Some("Archive/2026".to_owned()),
        }
    }

    #[test]
    fn command_and_status_round_trip_canonically() {
        let command = command();
        let bytes = encode_message_location_command(&command).expect("encode command");
        assert_eq!(
            decode_message_location_command(&bytes).expect("decode command"),
            command
        );
        let status = MailMessageLocationOperationStatusV1 {
            operation_id: "location-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            kind: MailMessageLocationKindV1::Move,
            target_folder_id: Some("Archive/2026".to_owned()),
            outcome: MailMessageLocationOperationOutcomeV1::Succeeded,
            requested_at_unix_seconds: 10,
            completed_at_unix_seconds: Some(11),
            projection_revision: Some(2),
        };
        let bytes = encode_message_location_status_response(Some(&status)).expect("encode status");
        assert_eq!(
            decode_message_location_status_response(&bytes).expect("decode status"),
            Some(status)
        );
    }

    #[test]
    fn decoder_rejects_unknown_kind_and_noncanonical_bytes() {
        let mut wire = wire::MailMessageLocationCommandV1 {
            operation_id: "location-operation-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            kind: 99,
            target_folder_id: None,
        };
        assert_eq!(
            decode_message_location_command(&wire.encode_to_vec()),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
        wire.kind = wire::MailMessageLocationKindV1::MailMessageLocationKindArchive as i32;
        let mut bytes = wire.encode_to_vec();
        bytes.extend_from_slice(&[0x98, 0x06, 0x00]);
        assert_eq!(
            decode_message_location_command(&bytes),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
