//! Typed Mail-owned account credential binding and sanitized status.

const MAX_CONNECTION_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MailCredentialPurposeV1 {
    ImapPassword,
    SmtpPassword,
    GmailAccessToken,
    GmailRefreshCredential,
}

impl MailCredentialPurposeV1 {
    #[must_use]
    pub const fn bindable_by_client(self) -> bool {
        matches!(self, Self::ImapPassword | Self::SmtpPassword)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCredentialBindingStateV1 {
    Unconfigured,
    PendingRestart,
    Active,
    Retired,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountReadinessV1 {
    ConfigurationOnly,
    PendingRestart,
    Ready,
    Retired,
    Deleted,
    Degraded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailConnectorProfileV1 {
    Imap,
    ImapSmtp,
    Gmail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailProviderPathReadinessV1 {
    NotConfigured,
    CredentialRequired,
    PendingRestart,
    Ready,
    Retired,
    Deleted,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailBindCredentialRequestV1 {
    pub connection_id: String,
    pub purpose: MailCredentialPurposeV1,
    pub expected_binding_revision: u64,
    pub credential_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCredentialBindingReceiptV1 {
    pub connection_id: String,
    pub purpose: MailCredentialPurposeV1,
    pub binding_revision: u64,
    pub state: MailCredentialBindingStateV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountStatusRequestV1 {
    pub connection_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCredentialBindingStatusV1 {
    pub purpose: MailCredentialPurposeV1,
    pub state: MailCredentialBindingStateV1,
    pub binding_revision: Option<u64>,
    pub credential_revision: Option<u64>,
    pub applied_runtime_generation: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountStatusV1 {
    pub connection_id: String,
    pub settings_revision: u64,
    pub runtime_generation: u64,
    pub readiness: MailAccountReadinessV1,
    pub connector_profile: MailConnectorProfileV1,
    pub sync_readiness: MailProviderPathReadinessV1,
    pub delivery_readiness: MailProviderPathReadinessV1,
    pub bindings: Vec<MailCredentialBindingStatusV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountValidationErrorV1 {
    Invalid,
}

pub fn validate_bind_credential_request(
    request: &MailBindCredentialRequestV1,
) -> Result<(), MailAccountValidationErrorV1> {
    (valid_connection_id(&request.connection_id)
        && request.purpose.bindable_by_client()
        && request.credential_revision > 0)
        .then_some(())
        .ok_or(MailAccountValidationErrorV1::Invalid)
}

pub fn validate_binding_receipt(
    receipt: &MailCredentialBindingReceiptV1,
) -> Result<(), MailAccountValidationErrorV1> {
    (valid_connection_id(&receipt.connection_id)
        && receipt.purpose.bindable_by_client()
        && receipt.binding_revision > 0
        && receipt.state == MailCredentialBindingStateV1::PendingRestart)
        .then_some(())
        .ok_or(MailAccountValidationErrorV1::Invalid)
}

pub fn validate_account_status_request(
    request: &MailAccountStatusRequestV1,
) -> Result<(), MailAccountValidationErrorV1> {
    valid_connection_id(&request.connection_id)
        .then_some(())
        .ok_or(MailAccountValidationErrorV1::Invalid)
}

pub fn validate_account_status(
    status: &MailAccountStatusV1,
) -> Result<(), MailAccountValidationErrorV1> {
    if !valid_connection_id(&status.connection_id)
        || status.settings_revision == 0
        || status.runtime_generation == 0
        || status.bindings.len() > 4
    {
        return Err(MailAccountValidationErrorV1::Invalid);
    }
    let mut purposes = status
        .bindings
        .iter()
        .map(|binding| binding.purpose)
        .collect::<Vec<_>>();
    purposes.sort_unstable();
    purposes.dedup();
    (purposes.len() == status.bindings.len() && status.bindings.iter().all(valid_binding_status))
        .then_some(())
        .ok_or(MailAccountValidationErrorV1::Invalid)
}

fn valid_binding_status(status: &MailCredentialBindingStatusV1) -> bool {
    match status.state {
        MailCredentialBindingStateV1::Unconfigured => {
            status.binding_revision.is_none()
                && status.credential_revision.is_none()
                && status.applied_runtime_generation.is_none()
        }
        MailCredentialBindingStateV1::PendingRestart => {
            status.binding_revision.is_some_and(|value| value > 0)
                && status.credential_revision.is_some_and(|value| value > 0)
                && status.applied_runtime_generation.is_none()
        }
        MailCredentialBindingStateV1::Active => {
            status.credential_revision.is_some_and(|value| value > 0)
                && status
                    .applied_runtime_generation
                    .is_some_and(|value| value > 0)
                && (status.purpose.bindable_by_client()
                    == status.binding_revision.is_some_and(|value| value > 0))
        }
        MailCredentialBindingStateV1::Retired | MailCredentialBindingStateV1::Deleted => {
            status.credential_revision.is_some_and(|value| value > 0)
                && status.applied_runtime_generation.is_none()
        }
    }
}

fn valid_connection_id(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_CONNECTION_ID_BYTES && !value.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_binding_accepts_only_basic_mail_purposes_without_secret_carriers() {
        let request = MailBindCredentialRequestV1 {
            connection_id: "mail-account".to_owned(),
            purpose: MailCredentialPurposeV1::ImapPassword,
            expected_binding_revision: 0,
            credential_revision: 1,
        };
        assert_eq!(validate_bind_credential_request(&request), Ok(()));
        assert!(
            validate_bind_credential_request(&MailBindCredentialRequestV1 {
                purpose: MailCredentialPurposeV1::GmailAccessToken,
                ..request
            })
            .is_err()
        );
    }
}
