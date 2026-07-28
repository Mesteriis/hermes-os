use prost::Message;

use crate::{
    client_wire::MailClientWireErrorV1,
    message_permanent_delete::{
        MailMessagePermanentDeleteAcceptedV1, MailMessagePermanentDeleteCommandV1,
        MailMessagePermanentDeleteConfirmationV1, MailMessagePermanentDeleteOperationOutcomeV1,
        MailMessagePermanentDeleteOperationStatusV1, MailMessagePermanentDeleteStatusRequestV1,
        validate_message_permanent_delete_accepted, validate_message_permanent_delete_command,
        validate_message_permanent_delete_status, validate_message_permanent_delete_status_request,
    },
    message_permanent_delete_wire_generated as wire,
};

pub fn encode_message_permanent_delete_command(
    command: &MailMessagePermanentDeleteCommandV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_permanent_delete_command(command)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessagePermanentDeleteCommandV1 {
        operation_id: command.operation_id.clone(),
        connection_id: command.connection_id.clone(),
        message_id: command.message_id.clone(),
        expected_projection_revision: command.expected_projection_revision,
        confirmation: confirmation_to_wire(command.confirmation),
    }
    .encode_to_vec())
}

pub fn decode_message_permanent_delete_command(
    bytes: &[u8],
) -> Result<MailMessagePermanentDeleteCommandV1, MailClientWireErrorV1> {
    let value = wire::MailMessagePermanentDeleteCommandV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let command = MailMessagePermanentDeleteCommandV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        expected_projection_revision: value.expected_projection_revision,
        confirmation: confirmation_from_wire(value.confirmation)?,
    };
    validate_message_permanent_delete_command(&command)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_permanent_delete_command(&command)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(command)
}

pub fn encode_message_permanent_delete_accepted(
    accepted: &MailMessagePermanentDeleteAcceptedV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_permanent_delete_accepted(accepted)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessagePermanentDeleteAcceptedV1 {
        operation_id: accepted.operation_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_permanent_delete_accepted(
    bytes: &[u8],
) -> Result<MailMessagePermanentDeleteAcceptedV1, MailClientWireErrorV1> {
    let value = wire::MailMessagePermanentDeleteAcceptedV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let accepted = MailMessagePermanentDeleteAcceptedV1 {
        operation_id: value.operation_id,
    };
    validate_message_permanent_delete_accepted(&accepted)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_permanent_delete_accepted(&accepted)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(accepted)
}

pub fn encode_message_permanent_delete_status_request(
    request: &MailMessagePermanentDeleteStatusRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_message_permanent_delete_status_request(request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessagePermanentDeleteStatusRequestV1 {
        operation_id: request.operation_id.clone(),
        connection_id: request.connection_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_message_permanent_delete_status_request(
    bytes: &[u8],
) -> Result<MailMessagePermanentDeleteStatusRequestV1, MailClientWireErrorV1> {
    let value = wire::MailMessagePermanentDeleteStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailMessagePermanentDeleteStatusRequestV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
    };
    validate_message_permanent_delete_status_request(&request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if encode_message_permanent_delete_status_request(&request)? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

pub fn encode_message_permanent_delete_status_response(
    status: Option<&MailMessagePermanentDeleteOperationStatusV1>,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    let status = status.map(status_to_wire).transpose()?;
    Ok(wire::MailMessagePermanentDeleteStatusResponseV1 { status }.encode_to_vec())
}

pub fn decode_message_permanent_delete_status_response(
    bytes: &[u8],
) -> Result<Option<MailMessagePermanentDeleteOperationStatusV1>, MailClientWireErrorV1> {
    let status = wire::MailMessagePermanentDeleteStatusResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .status
        .map(status_from_wire)
        .transpose()?;
    if encode_message_permanent_delete_status_response(status.as_ref())? != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(status)
}

fn status_to_wire(
    status: &MailMessagePermanentDeleteOperationStatusV1,
) -> Result<wire::MailMessagePermanentDeleteOperationStatusV1, MailClientWireErrorV1> {
    validate_message_permanent_delete_status(status)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailMessagePermanentDeleteOperationStatusV1 {
        operation_id: status.operation_id.clone(),
        connection_id: status.connection_id.clone(),
        message_id: status.message_id.clone(),
        expected_projection_revision: status.expected_projection_revision,
        confirmation: confirmation_to_wire(status.confirmation),
        outcome: outcome_to_wire(status.outcome),
        requested_at_unix_seconds: status.requested_at_unix_seconds,
        completed_at_unix_seconds: status.completed_at_unix_seconds,
        deletion_projection_revision: status.deletion_projection_revision,
    })
}

fn status_from_wire(
    value: wire::MailMessagePermanentDeleteOperationStatusV1,
) -> Result<MailMessagePermanentDeleteOperationStatusV1, MailClientWireErrorV1> {
    let status = MailMessagePermanentDeleteOperationStatusV1 {
        operation_id: value.operation_id,
        connection_id: value.connection_id,
        message_id: value.message_id,
        expected_projection_revision: value.expected_projection_revision,
        confirmation: confirmation_from_wire(value.confirmation)?,
        outcome: outcome_from_wire(value.outcome)?,
        requested_at_unix_seconds: value.requested_at_unix_seconds,
        completed_at_unix_seconds: value.completed_at_unix_seconds,
        deletion_projection_revision: value.deletion_projection_revision,
    };
    validate_message_permanent_delete_status(&status)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(status)
}

const fn confirmation_to_wire(confirmation: MailMessagePermanentDeleteConfirmationV1) -> i32 {
    match confirmation {
        MailMessagePermanentDeleteConfirmationV1::Confirmed => wire::
            MailMessagePermanentDeleteConfirmationV1::
            MailMessagePermanentDeleteConfirmationConfirmed as i32,
    }
}

fn confirmation_from_wire(
    value: i32,
) -> Result<MailMessagePermanentDeleteConfirmationV1, MailClientWireErrorV1> {
    match wire::MailMessagePermanentDeleteConfirmationV1::try_from(value)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::MailMessagePermanentDeleteConfirmationV1::
            MailMessagePermanentDeleteConfirmationConfirmed => {
                Ok(MailMessagePermanentDeleteConfirmationV1::Confirmed)
            }
        wire::MailMessagePermanentDeleteConfirmationV1::
            MailMessagePermanentDeleteConfirmationUnspecified => {
                Err(MailClientWireErrorV1::InvalidPayload)
            }
    }
}

const fn outcome_to_wire(outcome: MailMessagePermanentDeleteOperationOutcomeV1) -> i32 {
    use wire::MailMessagePermanentDeleteOperationOutcomeV1 as Wire;
    match outcome {
        MailMessagePermanentDeleteOperationOutcomeV1::Pending => {
            Wire::MailMessagePermanentDeleteOperationOutcomePending as i32
        }
        MailMessagePermanentDeleteOperationOutcomeV1::Succeeded => {
            Wire::MailMessagePermanentDeleteOperationOutcomeSucceeded as i32
        }
        MailMessagePermanentDeleteOperationOutcomeV1::Rejected => {
            Wire::MailMessagePermanentDeleteOperationOutcomeRejected as i32
        }
        MailMessagePermanentDeleteOperationOutcomeV1::Unsupported => {
            Wire::MailMessagePermanentDeleteOperationOutcomeUnsupported as i32
        }
        MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired => {
            Wire::MailMessagePermanentDeleteOperationOutcomeReauthorizationRequired as i32
        }
        MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown => {
            Wire::MailMessagePermanentDeleteOperationOutcomeUnknown as i32
        }
    }
}

fn outcome_from_wire(
    value: i32,
) -> Result<MailMessagePermanentDeleteOperationOutcomeV1, MailClientWireErrorV1> {
    use wire::MailMessagePermanentDeleteOperationOutcomeV1 as Wire;
    match Wire::try_from(value).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Wire::MailMessagePermanentDeleteOperationOutcomePending => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::Pending)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeSucceeded => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::Succeeded)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeRejected => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::Rejected)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeUnsupported => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::Unsupported)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeReauthorizationRequired => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeUnknown => {
            Ok(MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown)
        }
        Wire::MailMessagePermanentDeleteOperationOutcomeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MailMessagePermanentDeleteCommandV1 {
        MailMessagePermanentDeleteCommandV1 {
            operation_id: "permanent-delete-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            expected_projection_revision: 7,
            confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
        }
    }

    #[test]
    fn command_and_status_round_trip_canonically() {
        let command = command();
        let bytes = encode_message_permanent_delete_command(&command).expect("encode command");
        assert_eq!(
            decode_message_permanent_delete_command(&bytes).expect("decode command"),
            command
        );
        let status = MailMessagePermanentDeleteOperationStatusV1 {
            operation_id: command.operation_id,
            connection_id: command.connection_id,
            message_id: command.message_id,
            expected_projection_revision: command.expected_projection_revision,
            confirmation: command.confirmation,
            outcome: MailMessagePermanentDeleteOperationOutcomeV1::Succeeded,
            requested_at_unix_seconds: 10,
            completed_at_unix_seconds: Some(11),
            deletion_projection_revision: Some(8),
        };
        let bytes =
            encode_message_permanent_delete_status_response(Some(&status)).expect("encode status");
        assert_eq!(
            decode_message_permanent_delete_status_response(&bytes).expect("decode status"),
            Some(status)
        );
    }

    #[test]
    fn decoder_rejects_unspecified_confirmation() {
        let wire = wire::MailMessagePermanentDeleteCommandV1 {
            operation_id: "permanent-delete-1".to_owned(),
            connection_id: "mail-account-1".to_owned(),
            message_id: "message-1".to_owned(),
            expected_projection_revision: 7,
            confirmation: 0,
        };
        assert_eq!(
            decode_message_permanent_delete_command(&wire.encode_to_vec()),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
