//! Protobuf mapping for Mail account lifecycle commands and receipts.

use prost::Message;

use crate::{
    account::MailCredentialPurposeV1,
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleReceiptV1,
        MailAccountLifecycleRetryV1, MailAccountLifecycleStateV1,
        MailAccountLifecycleStatusRequestV1, MailCredentialLifecycleProgressV1,
        MailCredentialLifecycleStateV1, validate_lifecycle_command, validate_lifecycle_receipt,
        validate_lifecycle_retry, validate_lifecycle_status_request,
    },
    account_lifecycle_wire_generated as wire,
    client_wire::MailClientWireErrorV1,
};

pub fn encode_command(
    command: &MailAccountLifecycleCommandV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_lifecycle_command(command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountLifecycleCommandV1 {
        operation_id: command.operation_id.clone(),
        connection_id: command.connection_id.clone(),
        expected_lifecycle_revision: command.expected_lifecycle_revision,
    }
    .encode_to_vec())
}

pub fn decode_command(
    bytes: &[u8],
) -> Result<MailAccountLifecycleCommandV1, MailClientWireErrorV1> {
    let command = wire::MailAccountLifecycleCommandV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let command = MailAccountLifecycleCommandV1 {
        operation_id: command.operation_id,
        connection_id: command.connection_id,
        expected_lifecycle_revision: command.expected_lifecycle_revision,
    };
    validate_lifecycle_command(&command).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(command)
}

pub fn encode_retry(retry: &MailAccountLifecycleRetryV1) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_lifecycle_retry(retry).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountLifecycleRetryV1 {
        operation_id: retry.operation_id.clone(),
        connection_id: retry.connection_id.clone(),
        expected_lifecycle_revision: retry.expected_lifecycle_revision,
    }
    .encode_to_vec())
}

pub fn decode_retry(bytes: &[u8]) -> Result<MailAccountLifecycleRetryV1, MailClientWireErrorV1> {
    let retry = wire::MailAccountLifecycleRetryV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let retry = MailAccountLifecycleRetryV1 {
        operation_id: retry.operation_id,
        connection_id: retry.connection_id,
        expected_lifecycle_revision: retry.expected_lifecycle_revision,
    };
    validate_lifecycle_retry(&retry).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(retry)
}

pub fn encode_status_request(
    request: &MailAccountLifecycleStatusRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_lifecycle_status_request(request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountLifecycleStatusRequestV1 {
        operation_id: request.operation_id.clone(),
        connection_id: request.connection_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_status_request(
    bytes: &[u8],
) -> Result<MailAccountLifecycleStatusRequestV1, MailClientWireErrorV1> {
    let request = wire::MailAccountLifecycleStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailAccountLifecycleStatusRequestV1 {
        operation_id: request.operation_id,
        connection_id: request.connection_id,
    };
    validate_lifecycle_status_request(&request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(request)
}

pub fn encode_receipt(
    receipt: &MailAccountLifecycleReceiptV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_lifecycle_receipt(receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountLifecycleReceiptV1 {
        operation_id: receipt.operation_id.clone(),
        connection_id: receipt.connection_id.clone(),
        action: action_to_wire(receipt.action),
        lifecycle_revision: receipt.lifecycle_revision,
        state: lifecycle_state_to_wire(receipt.state),
        credential: receipt
            .credentials
            .iter()
            .map(|progress| wire::MailCredentialLifecycleProgressV1 {
                purpose: purpose_to_wire(progress.purpose),
                state: credential_state_to_wire(progress.state),
                binding_revision: progress.binding_revision,
                credential_revision: progress.credential_revision,
            })
            .collect(),
    }
    .encode_to_vec())
}

pub fn decode_receipt(
    bytes: &[u8],
) -> Result<MailAccountLifecycleReceiptV1, MailClientWireErrorV1> {
    let receipt = wire::MailAccountLifecycleReceiptV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let receipt = MailAccountLifecycleReceiptV1 {
        operation_id: receipt.operation_id,
        connection_id: receipt.connection_id,
        action: action_from_wire(receipt.action)?,
        lifecycle_revision: receipt.lifecycle_revision,
        state: lifecycle_state_from_wire(receipt.state)?,
        credentials: receipt
            .credential
            .into_iter()
            .map(|progress| {
                Ok(MailCredentialLifecycleProgressV1 {
                    purpose: purpose_from_wire(progress.purpose)?,
                    state: credential_state_from_wire(progress.state)?,
                    binding_revision: progress.binding_revision,
                    credential_revision: progress.credential_revision,
                })
            })
            .collect::<Result<_, MailClientWireErrorV1>>()?,
    };
    validate_lifecycle_receipt(&receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(receipt)
}

fn action_to_wire(action: MailAccountLifecycleActionV1) -> i32 {
    use wire::MailAccountLifecycleActionV1 as Action;
    match action {
        MailAccountLifecycleActionV1::Retire => Action::MailAccountLifecycleActionRetire as i32,
        MailAccountLifecycleActionV1::Delete => Action::MailAccountLifecycleActionDelete as i32,
    }
}

fn purpose_to_wire(purpose: MailCredentialPurposeV1) -> i32 {
    use wire::MailLifecycleCredentialPurposeV1 as Purpose;
    match purpose {
        MailCredentialPurposeV1::ImapPassword => {
            Purpose::MailLifecycleCredentialPurposeImapPassword as i32
        }
        MailCredentialPurposeV1::SmtpPassword => {
            Purpose::MailLifecycleCredentialPurposeSmtpPassword as i32
        }
        MailCredentialPurposeV1::GmailAccessToken => {
            Purpose::MailLifecycleCredentialPurposeGmailAccessToken as i32
        }
        MailCredentialPurposeV1::GmailRefreshCredential => {
            Purpose::MailLifecycleCredentialPurposeGmailRefreshCredential as i32
        }
    }
}

fn purpose_from_wire(purpose: i32) -> Result<MailCredentialPurposeV1, MailClientWireErrorV1> {
    use wire::MailLifecycleCredentialPurposeV1 as Purpose;
    match Purpose::try_from(purpose).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Purpose::MailLifecycleCredentialPurposeImapPassword => {
            Ok(MailCredentialPurposeV1::ImapPassword)
        }
        Purpose::MailLifecycleCredentialPurposeSmtpPassword => {
            Ok(MailCredentialPurposeV1::SmtpPassword)
        }
        Purpose::MailLifecycleCredentialPurposeGmailAccessToken => {
            Ok(MailCredentialPurposeV1::GmailAccessToken)
        }
        Purpose::MailLifecycleCredentialPurposeGmailRefreshCredential => {
            Ok(MailCredentialPurposeV1::GmailRefreshCredential)
        }
        Purpose::MailLifecycleCredentialPurposeUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

fn action_from_wire(action: i32) -> Result<MailAccountLifecycleActionV1, MailClientWireErrorV1> {
    use wire::MailAccountLifecycleActionV1 as Action;
    match Action::try_from(action).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Action::MailAccountLifecycleActionRetire => Ok(MailAccountLifecycleActionV1::Retire),
        Action::MailAccountLifecycleActionDelete => Ok(MailAccountLifecycleActionV1::Delete),
        Action::MailAccountLifecycleActionUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn lifecycle_state_to_wire(state: MailAccountLifecycleStateV1) -> i32 {
    use wire::MailAccountLifecycleStateV1 as State;
    match state {
        MailAccountLifecycleStateV1::Pending => State::MailAccountLifecycleStatePending as i32,
        MailAccountLifecycleStateV1::Completed => State::MailAccountLifecycleStateCompleted as i32,
        MailAccountLifecycleStateV1::Rejected => State::MailAccountLifecycleStateRejected as i32,
        MailAccountLifecycleStateV1::OutcomeUnknown => {
            State::MailAccountLifecycleStateOutcomeUnknown as i32
        }
    }
}

fn lifecycle_state_from_wire(
    state: i32,
) -> Result<MailAccountLifecycleStateV1, MailClientWireErrorV1> {
    use wire::MailAccountLifecycleStateV1 as State;
    match State::try_from(state).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        State::MailAccountLifecycleStatePending => Ok(MailAccountLifecycleStateV1::Pending),
        State::MailAccountLifecycleStateCompleted => Ok(MailAccountLifecycleStateV1::Completed),
        State::MailAccountLifecycleStateRejected => Ok(MailAccountLifecycleStateV1::Rejected),
        State::MailAccountLifecycleStateOutcomeUnknown => {
            Ok(MailAccountLifecycleStateV1::OutcomeUnknown)
        }
        State::MailAccountLifecycleStateUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn credential_state_to_wire(state: MailCredentialLifecycleStateV1) -> i32 {
    use wire::MailCredentialLifecycleStateV1 as State;
    match state {
        MailCredentialLifecycleStateV1::Pending => {
            State::MailCredentialLifecycleStatePending as i32
        }
        MailCredentialLifecycleStateV1::Completed => {
            State::MailCredentialLifecycleStateCompleted as i32
        }
        MailCredentialLifecycleStateV1::Rejected => {
            State::MailCredentialLifecycleStateRejected as i32
        }
        MailCredentialLifecycleStateV1::OutcomeUnknown => {
            State::MailCredentialLifecycleStateOutcomeUnknown as i32
        }
    }
}

fn credential_state_from_wire(
    state: i32,
) -> Result<MailCredentialLifecycleStateV1, MailClientWireErrorV1> {
    use wire::MailCredentialLifecycleStateV1 as State;
    match State::try_from(state).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        State::MailCredentialLifecycleStatePending => Ok(MailCredentialLifecycleStateV1::Pending),
        State::MailCredentialLifecycleStateCompleted => {
            Ok(MailCredentialLifecycleStateV1::Completed)
        }
        State::MailCredentialLifecycleStateRejected => Ok(MailCredentialLifecycleStateV1::Rejected),
        State::MailCredentialLifecycleStateOutcomeUnknown => {
            Ok(MailCredentialLifecycleStateV1::OutcomeUnknown)
        }
        State::MailCredentialLifecycleStateUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::MailCredentialPurposeV1;

    use super::*;

    #[test]
    fn lifecycle_receipt_round_trips_without_secret_carriers() {
        let receipt = MailAccountLifecycleReceiptV1 {
            operation_id: "retire-1".to_owned(),
            connection_id: "mail-account".to_owned(),
            action: MailAccountLifecycleActionV1::Retire,
            lifecycle_revision: 2,
            state: MailAccountLifecycleStateV1::OutcomeUnknown,
            credentials: vec![MailCredentialLifecycleProgressV1 {
                purpose: MailCredentialPurposeV1::ImapPassword,
                state: MailCredentialLifecycleStateV1::OutcomeUnknown,
                binding_revision: Some(3),
                credential_revision: 4,
            }],
        };

        assert_eq!(
            decode_receipt(&encode_receipt(&receipt).expect("encode")),
            Ok(receipt)
        );
    }
}
