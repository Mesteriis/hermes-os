//! Typed Zulip account credential-binding lifecycle.

const MAX_ACCOUNT_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipCredentialBindingStateV1 {
    Unconfigured,
    PendingRestart,
    Active,
    Retired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipAccountLifecycleCommandV1 {
    BindCredential {
        account_id: String,
        expected_binding_revision: u64,
        credential_revision: u64,
    },
    RetireAccount {
        account_id: String,
        expected_binding_revision: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipAccountLifecycleReceiptV1 {
    pub account_id: String,
    pub binding_revision: u64,
    pub state: ZulipCredentialBindingStateV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipAccountLifecycleValidationErrorV1 {
    Invalid,
}

pub fn validate_account_lifecycle_command(
    command: &ZulipAccountLifecycleCommandV1,
) -> Result<(), ZulipAccountLifecycleValidationErrorV1> {
    match command {
        ZulipAccountLifecycleCommandV1::BindCredential {
            account_id,
            credential_revision,
            ..
        } => valid_account_id(account_id)
            .then_some(())
            .filter(|_| *credential_revision > 0)
            .ok_or(ZulipAccountLifecycleValidationErrorV1::Invalid),
        ZulipAccountLifecycleCommandV1::RetireAccount {
            account_id,
            expected_binding_revision,
        } => valid_account_id(account_id)
            .then_some(())
            .filter(|_| *expected_binding_revision > 0)
            .ok_or(ZulipAccountLifecycleValidationErrorV1::Invalid),
    }
}

pub fn validate_account_lifecycle_receipt(
    receipt: &ZulipAccountLifecycleReceiptV1,
) -> Result<(), ZulipAccountLifecycleValidationErrorV1> {
    (valid_account_id(&receipt.account_id)
        && receipt.binding_revision > 0
        && receipt.state != ZulipCredentialBindingStateV1::Unconfigured)
        .then_some(())
        .ok_or(ZulipAccountLifecycleValidationErrorV1::Invalid)
}

fn valid_account_id(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_ACCOUNT_ID_BYTES && !value.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_binding_is_revisioned_and_contains_no_secret_carrier() {
        assert_eq!(
            validate_account_lifecycle_command(&ZulipAccountLifecycleCommandV1::BindCredential {
                account_id: "account".to_owned(),
                expected_binding_revision: 0,
                credential_revision: 1,
            }),
            Ok(())
        );
        assert!(
            validate_account_lifecycle_command(&ZulipAccountLifecycleCommandV1::BindCredential {
                account_id: "account".to_owned(),
                expected_binding_revision: 0,
                credential_revision: 0,
            })
            .is_err()
        );
    }
}
