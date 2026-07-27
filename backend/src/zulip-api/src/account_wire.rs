//! Protobuf mapping for the Zulip account credential-binding lifecycle.

use prost::Message;

use crate::{
    account::{
        ZulipAccountLifecycleCommandV1, ZulipAccountLifecycleReceiptV1,
        ZulipCredentialBindingStateV1, validate_account_lifecycle_command,
        validate_account_lifecycle_receipt,
    },
    account_wire_generated as wire,
    client_wire::ZulipClientWireErrorV1,
};

pub fn encode_account_lifecycle_command(
    command: &ZulipAccountLifecycleCommandV1,
) -> Result<Vec<u8>, ZulipClientWireErrorV1> {
    validate_account_lifecycle_command(command)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    use wire::zulip_account_lifecycle_command_v1::Command;
    let command = match command {
        ZulipAccountLifecycleCommandV1::BindCredential {
            account_id,
            expected_binding_revision,
            credential_revision,
        } => Command::BindCredential(wire::ZulipBindCredentialV1 {
            account_id: account_id.clone(),
            expected_binding_revision: *expected_binding_revision,
            credential_revision: *credential_revision,
        }),
        ZulipAccountLifecycleCommandV1::RetireAccount {
            account_id,
            expected_binding_revision,
        } => Command::RetireAccount(wire::ZulipRetireAccountV1 {
            account_id: account_id.clone(),
            expected_binding_revision: *expected_binding_revision,
        }),
    };
    Ok(wire::ZulipAccountLifecycleCommandV1 {
        command: Some(command),
    }
    .encode_to_vec())
}

pub fn decode_account_lifecycle_command(
    bytes: &[u8],
) -> Result<ZulipAccountLifecycleCommandV1, ZulipClientWireErrorV1> {
    use wire::zulip_account_lifecycle_command_v1::Command;
    let command = wire::ZulipAccountLifecycleCommandV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?
        .command
        .ok_or(ZulipClientWireErrorV1::MissingVariant)?;
    let command = match command {
        Command::BindCredential(value) => ZulipAccountLifecycleCommandV1::BindCredential {
            account_id: value.account_id,
            expected_binding_revision: value.expected_binding_revision,
            credential_revision: value.credential_revision,
        },
        Command::RetireAccount(value) => ZulipAccountLifecycleCommandV1::RetireAccount {
            account_id: value.account_id,
            expected_binding_revision: value.expected_binding_revision,
        },
    };
    validate_account_lifecycle_command(&command)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(command)
}

pub fn encode_account_lifecycle_receipt(
    receipt: &ZulipAccountLifecycleReceiptV1,
) -> Result<Vec<u8>, ZulipClientWireErrorV1> {
    validate_account_lifecycle_receipt(receipt)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(wire::ZulipAccountLifecycleReceiptV1 {
        account_id: receipt.account_id.clone(),
        binding_revision: receipt.binding_revision,
        state: state_to_wire(receipt.state),
    }
    .encode_to_vec())
}

pub fn decode_account_lifecycle_receipt(
    bytes: &[u8],
) -> Result<ZulipAccountLifecycleReceiptV1, ZulipClientWireErrorV1> {
    let receipt = wire::ZulipAccountLifecycleReceiptV1::decode(bytes)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    let receipt = ZulipAccountLifecycleReceiptV1 {
        account_id: receipt.account_id,
        binding_revision: receipt.binding_revision,
        state: state_from_wire(receipt.state)?,
    };
    validate_account_lifecycle_receipt(&receipt)
        .map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    Ok(receipt)
}

pub(crate) const fn state_to_wire(state: ZulipCredentialBindingStateV1) -> i32 {
    use wire::ZulipCredentialBindingStateV1 as State;
    match state {
        ZulipCredentialBindingStateV1::Unconfigured => {
            State::ZulipCredentialBindingStateUnconfigured as i32
        }
        ZulipCredentialBindingStateV1::PendingRestart => {
            State::ZulipCredentialBindingStatePendingRestart as i32
        }
        ZulipCredentialBindingStateV1::Active => State::ZulipCredentialBindingStateActive as i32,
        ZulipCredentialBindingStateV1::Retired => State::ZulipCredentialBindingStateRetired as i32,
    }
}

pub(crate) fn state_from_wire(
    value: i32,
) -> Result<ZulipCredentialBindingStateV1, ZulipClientWireErrorV1> {
    use wire::ZulipCredentialBindingStateV1 as State;
    match State::try_from(value).map_err(|_| ZulipClientWireErrorV1::InvalidPayload)? {
        State::ZulipCredentialBindingStateUnconfigured => {
            Ok(ZulipCredentialBindingStateV1::Unconfigured)
        }
        State::ZulipCredentialBindingStatePendingRestart => {
            Ok(ZulipCredentialBindingStateV1::PendingRestart)
        }
        State::ZulipCredentialBindingStateActive => Ok(ZulipCredentialBindingStateV1::Active),
        State::ZulipCredentialBindingStateRetired => Ok(ZulipCredentialBindingStateV1::Retired),
        State::ZulipCredentialBindingStateUnspecified => {
            Err(ZulipClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_lifecycle_round_trips_without_secret_bytes() {
        let command = ZulipAccountLifecycleCommandV1::BindCredential {
            account_id: "account".to_owned(),
            expected_binding_revision: 2,
            credential_revision: 3,
        };
        assert_eq!(
            decode_account_lifecycle_command(
                &encode_account_lifecycle_command(&command).expect("encode")
            ),
            Ok(command)
        );
    }
}
