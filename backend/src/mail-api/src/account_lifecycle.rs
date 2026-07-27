//! Typed Mail-owned account retire/delete lifecycle contracts.

use crate::account::MailCredentialPurposeV1;

const MAX_IDENTIFIER_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountLifecycleActionV1 {
    Retire,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountLifecycleStateV1 {
    Pending,
    Completed,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCredentialLifecycleStateV1 {
    Pending,
    Completed,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub expected_lifecycle_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleRetryV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub expected_lifecycle_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleStatusRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCredentialLifecycleProgressV1 {
    pub purpose: MailCredentialPurposeV1,
    pub state: MailCredentialLifecycleStateV1,
    pub binding_revision: Option<u64>,
    pub credential_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleReceiptV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub action: MailAccountLifecycleActionV1,
    pub lifecycle_revision: u64,
    pub state: MailAccountLifecycleStateV1,
    pub credentials: Vec<MailCredentialLifecycleProgressV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountLifecycleValidationErrorV1 {
    Invalid,
}

pub fn validate_lifecycle_command(
    command: &MailAccountLifecycleCommandV1,
) -> Result<(), MailAccountLifecycleValidationErrorV1> {
    (valid_identifier(&command.operation_id) && valid_identifier(&command.connection_id))
        .then_some(())
        .ok_or(MailAccountLifecycleValidationErrorV1::Invalid)
}

pub fn validate_lifecycle_retry(
    retry: &MailAccountLifecycleRetryV1,
) -> Result<(), MailAccountLifecycleValidationErrorV1> {
    (valid_identifier(&retry.operation_id)
        && valid_identifier(&retry.connection_id)
        && retry.expected_lifecycle_revision > 0)
        .then_some(())
        .ok_or(MailAccountLifecycleValidationErrorV1::Invalid)
}

pub fn validate_lifecycle_status_request(
    request: &MailAccountLifecycleStatusRequestV1,
) -> Result<(), MailAccountLifecycleValidationErrorV1> {
    (valid_identifier(&request.operation_id) && valid_identifier(&request.connection_id))
        .then_some(())
        .ok_or(MailAccountLifecycleValidationErrorV1::Invalid)
}

pub fn validate_lifecycle_receipt(
    receipt: &MailAccountLifecycleReceiptV1,
) -> Result<(), MailAccountLifecycleValidationErrorV1> {
    if !valid_identifier(&receipt.operation_id)
        || !valid_identifier(&receipt.connection_id)
        || receipt.lifecycle_revision == 0
        || receipt.credentials.len() > 4
    {
        return Err(MailAccountLifecycleValidationErrorV1::Invalid);
    }
    let mut purposes = receipt
        .credentials
        .iter()
        .map(|progress| progress.purpose)
        .collect::<Vec<_>>();
    purposes.sort_unstable();
    purposes.dedup();
    if purposes.len() != receipt.credentials.len()
        || receipt.credentials.iter().any(|progress| {
            progress.credential_revision == 0
                || (progress.purpose.bindable_by_client()
                    != progress
                        .binding_revision
                        .is_some_and(|revision| revision > 0))
        })
    {
        return Err(MailAccountLifecycleValidationErrorV1::Invalid);
    }
    let expected = aggregate_lifecycle_state(&receipt.credentials);
    (receipt.state == expected)
        .then_some(())
        .ok_or(MailAccountLifecycleValidationErrorV1::Invalid)
}

#[must_use]
pub fn aggregate_lifecycle_state(
    credentials: &[MailCredentialLifecycleProgressV1],
) -> MailAccountLifecycleStateV1 {
    if credentials
        .iter()
        .any(|progress| progress.state == MailCredentialLifecycleStateV1::OutcomeUnknown)
    {
        return MailAccountLifecycleStateV1::OutcomeUnknown;
    }
    if credentials
        .iter()
        .any(|progress| progress.state == MailCredentialLifecycleStateV1::Rejected)
    {
        return MailAccountLifecycleStateV1::Rejected;
    }
    if credentials
        .iter()
        .any(|progress| progress.state == MailCredentialLifecycleStateV1::Pending)
    {
        return MailAccountLifecycleStateV1::Pending;
    }
    MailAccountLifecycleStateV1::Completed
}

fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_IDENTIFIER_BYTES && !value.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_keeps_any_unknown_outcome_retryable() {
        let progress = |state| MailCredentialLifecycleProgressV1 {
            purpose: MailCredentialPurposeV1::ImapPassword,
            state,
            binding_revision: Some(1),
            credential_revision: 2,
        };
        assert_eq!(
            aggregate_lifecycle_state(&[progress(MailCredentialLifecycleStateV1::OutcomeUnknown)]),
            MailAccountLifecycleStateV1::OutcomeUnknown
        );
        assert_eq!(
            aggregate_lifecycle_state(&[
                progress(MailCredentialLifecycleStateV1::Rejected),
                MailCredentialLifecycleProgressV1 {
                    purpose: MailCredentialPurposeV1::SmtpPassword,
                    ..progress(MailCredentialLifecycleStateV1::OutcomeUnknown)
                },
            ]),
            MailAccountLifecycleStateV1::OutcomeUnknown
        );
        assert_eq!(
            aggregate_lifecycle_state(&[progress(MailCredentialLifecycleStateV1::Rejected)]),
            MailAccountLifecycleStateV1::Rejected
        );
        assert_eq!(
            aggregate_lifecycle_state(&[progress(MailCredentialLifecycleStateV1::Pending)]),
            MailAccountLifecycleStateV1::Pending
        );
        assert_eq!(
            aggregate_lifecycle_state(&[]),
            MailAccountLifecycleStateV1::Completed
        );
    }
}
