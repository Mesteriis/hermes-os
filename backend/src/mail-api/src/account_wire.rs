//! Protobuf mapping for Mail account credential binding and status.

use prost::Message;

use crate::{
    account::{
        MailAccountCatalogRequestV1, MailAccountCatalogV1, MailAccountReadinessV1,
        MailAccountStatusRequestV1, MailAccountStatusV1, MailBindCredentialRequestV1,
        MailConnectorProfileV1, MailCredentialBindingReceiptV1, MailCredentialBindingStateV1,
        MailCredentialBindingStatusV1, MailCredentialPurposeV1, MailProviderPathReadinessV1,
        validate_account_catalog, validate_account_status, validate_account_status_request,
        validate_bind_credential_request, validate_binding_receipt,
    },
    account_wire_generated as wire,
    client_wire::MailClientWireErrorV1,
};

pub fn encode_bind_request(
    request: &MailBindCredentialRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_bind_credential_request(request).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailBindCredentialRequestV1 {
        connection_id: request.connection_id.clone(),
        purpose: purpose_to_wire(request.purpose),
        expected_binding_revision: request.expected_binding_revision,
        credential_revision: request.credential_revision,
    }
    .encode_to_vec())
}

pub fn decode_bind_request(
    bytes: &[u8],
) -> Result<MailBindCredentialRequestV1, MailClientWireErrorV1> {
    let request = wire::MailBindCredentialRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailBindCredentialRequestV1 {
        connection_id: request.connection_id,
        purpose: purpose_from_wire(request.purpose)?,
        expected_binding_revision: request.expected_binding_revision,
        credential_revision: request.credential_revision,
    };
    validate_bind_credential_request(&request)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(request)
}

pub fn encode_binding_receipt(
    receipt: &MailCredentialBindingReceiptV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_binding_receipt(receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailCredentialBindingReceiptV1 {
        connection_id: receipt.connection_id.clone(),
        purpose: purpose_to_wire(receipt.purpose),
        binding_revision: receipt.binding_revision,
        state: state_to_wire(receipt.state),
    }
    .encode_to_vec())
}

pub fn decode_binding_receipt(
    bytes: &[u8],
) -> Result<MailCredentialBindingReceiptV1, MailClientWireErrorV1> {
    let receipt = wire::MailCredentialBindingReceiptV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let receipt = MailCredentialBindingReceiptV1 {
        connection_id: receipt.connection_id,
        purpose: purpose_from_wire(receipt.purpose)?,
        binding_revision: receipt.binding_revision,
        state: state_from_wire(receipt.state)?,
    };
    validate_binding_receipt(&receipt).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(receipt)
}

pub fn encode_status_request(
    request: &MailAccountStatusRequestV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_account_status_request(request).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountStatusRequestV1 {
        connection_id: request.connection_id.clone(),
    }
    .encode_to_vec())
}

pub fn decode_status_request(
    bytes: &[u8],
) -> Result<MailAccountStatusRequestV1, MailClientWireErrorV1> {
    let request = wire::MailAccountStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailAccountStatusRequestV1 {
        connection_id: request.connection_id,
    };
    validate_account_status_request(&request).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(request)
}

pub fn encode_catalog_request(_: &MailAccountCatalogRequestV1) -> Vec<u8> {
    wire::MailAccountCatalogRequestV1 { major: 1 }.encode_to_vec()
}

pub fn decode_catalog_request(
    bytes: &[u8],
) -> Result<MailAccountCatalogRequestV1, MailClientWireErrorV1> {
    let request = wire::MailAccountCatalogRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if request.major != 1 || request.encode_to_vec() != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailAccountCatalogRequestV1)
}

pub fn encode_account_status(
    status: &MailAccountStatusV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_account_status(status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire_status(status).encode_to_vec())
}

pub fn decode_account_status(bytes: &[u8]) -> Result<MailAccountStatusV1, MailClientWireErrorV1> {
    let status = wire::MailAccountStatusV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    status_from_wire(status)
}

pub fn encode_account_catalog(
    catalog: &MailAccountCatalogV1,
) -> Result<Vec<u8>, MailClientWireErrorV1> {
    validate_account_catalog(catalog).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(wire::MailAccountCatalogV1 {
        accounts: catalog.accounts.iter().map(wire_status).collect(),
    }
    .encode_to_vec())
}

pub fn decode_account_catalog(bytes: &[u8]) -> Result<MailAccountCatalogV1, MailClientWireErrorV1> {
    let catalog = wire::MailAccountCatalogV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let catalog = MailAccountCatalogV1 {
        accounts: catalog
            .accounts
            .into_iter()
            .map(status_from_wire)
            .collect::<Result<_, _>>()?,
    };
    validate_account_catalog(&catalog).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(catalog)
}

fn status_from_wire(
    status: wire::MailAccountStatusV1,
) -> Result<MailAccountStatusV1, MailClientWireErrorV1> {
    let status = MailAccountStatusV1 {
        connection_id: status.connection_id,
        configuration_instance_id: status.configuration_instance_id,
        settings_revision: status.settings_revision,
        runtime_generation: status.runtime_generation,
        readiness: readiness_from_wire(status.readiness)?,
        connector_profile: connector_profile_from_wire(status.connector_profile)?,
        sync_readiness: provider_path_readiness_from_wire(status.sync_readiness)?,
        delivery_readiness: provider_path_readiness_from_wire(status.delivery_readiness)?,
        bindings: status
            .binding
            .into_iter()
            .map(binding_from_wire)
            .collect::<Result<_, _>>()?,
        lifecycle_revision: status.lifecycle_revision,
        lifecycle_operation_id: status.lifecycle_operation_id,
    };
    validate_account_status(&status).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    Ok(status)
}

fn wire_status(status: &MailAccountStatusV1) -> wire::MailAccountStatusV1 {
    wire::MailAccountStatusV1 {
        connection_id: status.connection_id.clone(),
        configuration_instance_id: status.configuration_instance_id.clone(),
        settings_revision: status.settings_revision,
        runtime_generation: status.runtime_generation,
        readiness: readiness_to_wire(status.readiness),
        connector_profile: connector_profile_to_wire(status.connector_profile),
        sync_readiness: provider_path_readiness_to_wire(status.sync_readiness),
        delivery_readiness: provider_path_readiness_to_wire(status.delivery_readiness),
        binding: status
            .bindings
            .iter()
            .map(|binding| wire::MailCredentialBindingStatusV1 {
                purpose: purpose_to_wire(binding.purpose),
                state: state_to_wire(binding.state),
                binding_revision: binding.binding_revision,
                credential_revision: binding.credential_revision,
                applied_runtime_generation: binding.applied_runtime_generation,
            })
            .collect(),
        lifecycle_revision: status.lifecycle_revision,
        lifecycle_operation_id: status.lifecycle_operation_id.clone(),
    }
}

fn binding_from_wire(
    binding: wire::MailCredentialBindingStatusV1,
) -> Result<MailCredentialBindingStatusV1, MailClientWireErrorV1> {
    Ok(MailCredentialBindingStatusV1 {
        purpose: purpose_from_wire(binding.purpose)?,
        state: state_from_wire(binding.state)?,
        binding_revision: binding.binding_revision,
        credential_revision: binding.credential_revision,
        applied_runtime_generation: binding.applied_runtime_generation,
    })
}

pub(crate) const fn purpose_to_wire(purpose: MailCredentialPurposeV1) -> i32 {
    use wire::MailCredentialPurposeV1 as Purpose;
    match purpose {
        MailCredentialPurposeV1::ImapPassword => Purpose::MailCredentialPurposeImapPassword as i32,
        MailCredentialPurposeV1::SmtpPassword => Purpose::MailCredentialPurposeSmtpPassword as i32,
        MailCredentialPurposeV1::GmailAccessToken => {
            Purpose::MailCredentialPurposeGmailAccessToken as i32
        }
        MailCredentialPurposeV1::GmailRefreshCredential => {
            Purpose::MailCredentialPurposeGmailRefreshCredential as i32
        }
        MailCredentialPurposeV1::IcloudCardDavPassword => {
            Purpose::MailCredentialPurposeIcloudCarddavPassword as i32
        }
    }
}

pub(crate) fn purpose_from_wire(
    purpose: i32,
) -> Result<MailCredentialPurposeV1, MailClientWireErrorV1> {
    use wire::MailCredentialPurposeV1 as Purpose;
    match Purpose::try_from(purpose).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Purpose::MailCredentialPurposeImapPassword => Ok(MailCredentialPurposeV1::ImapPassword),
        Purpose::MailCredentialPurposeSmtpPassword => Ok(MailCredentialPurposeV1::SmtpPassword),
        Purpose::MailCredentialPurposeGmailAccessToken => {
            Ok(MailCredentialPurposeV1::GmailAccessToken)
        }
        Purpose::MailCredentialPurposeGmailRefreshCredential => {
            Ok(MailCredentialPurposeV1::GmailRefreshCredential)
        }
        Purpose::MailCredentialPurposeIcloudCarddavPassword => {
            Ok(MailCredentialPurposeV1::IcloudCardDavPassword)
        }
        Purpose::MailCredentialPurposeUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn state_to_wire(state: MailCredentialBindingStateV1) -> i32 {
    use wire::MailCredentialBindingStateV1 as State;
    match state {
        MailCredentialBindingStateV1::Unconfigured => {
            State::MailCredentialBindingStateUnconfigured as i32
        }
        MailCredentialBindingStateV1::PendingRestart => {
            State::MailCredentialBindingStatePendingRestart as i32
        }
        MailCredentialBindingStateV1::Active => State::MailCredentialBindingStateActive as i32,
        MailCredentialBindingStateV1::Retired => State::MailCredentialBindingStateRetired as i32,
        MailCredentialBindingStateV1::Deleted => State::MailCredentialBindingStateDeleted as i32,
    }
}

fn state_from_wire(state: i32) -> Result<MailCredentialBindingStateV1, MailClientWireErrorV1> {
    use wire::MailCredentialBindingStateV1 as State;
    match State::try_from(state).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        State::MailCredentialBindingStateUnconfigured => {
            Ok(MailCredentialBindingStateV1::Unconfigured)
        }
        State::MailCredentialBindingStatePendingRestart => {
            Ok(MailCredentialBindingStateV1::PendingRestart)
        }
        State::MailCredentialBindingStateActive => Ok(MailCredentialBindingStateV1::Active),
        State::MailCredentialBindingStateRetired => Ok(MailCredentialBindingStateV1::Retired),
        State::MailCredentialBindingStateDeleted => Ok(MailCredentialBindingStateV1::Deleted),
        State::MailCredentialBindingStateUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn readiness_to_wire(readiness: MailAccountReadinessV1) -> i32 {
    use wire::MailAccountReadinessV1 as Readiness;
    match readiness {
        MailAccountReadinessV1::ConfigurationOnly => {
            Readiness::MailAccountReadinessConfigurationOnly as i32
        }
        MailAccountReadinessV1::PendingRestart => {
            Readiness::MailAccountReadinessPendingRestart as i32
        }
        MailAccountReadinessV1::Ready => Readiness::MailAccountReadinessReady as i32,
        MailAccountReadinessV1::Retired => Readiness::MailAccountReadinessRetired as i32,
        MailAccountReadinessV1::Deleted => Readiness::MailAccountReadinessDeleted as i32,
        MailAccountReadinessV1::Degraded => Readiness::MailAccountReadinessDegraded as i32,
    }
}

fn readiness_from_wire(readiness: i32) -> Result<MailAccountReadinessV1, MailClientWireErrorV1> {
    use wire::MailAccountReadinessV1 as Readiness;
    match Readiness::try_from(readiness).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Readiness::MailAccountReadinessConfigurationOnly => {
            Ok(MailAccountReadinessV1::ConfigurationOnly)
        }
        Readiness::MailAccountReadinessPendingRestart => Ok(MailAccountReadinessV1::PendingRestart),
        Readiness::MailAccountReadinessReady => Ok(MailAccountReadinessV1::Ready),
        Readiness::MailAccountReadinessRetired => Ok(MailAccountReadinessV1::Retired),
        Readiness::MailAccountReadinessDeleted => Ok(MailAccountReadinessV1::Deleted),
        Readiness::MailAccountReadinessDegraded => Ok(MailAccountReadinessV1::Degraded),
        Readiness::MailAccountReadinessUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn connector_profile_to_wire(profile: MailConnectorProfileV1) -> i32 {
    use wire::MailConnectorProfileV1 as Profile;
    match profile {
        MailConnectorProfileV1::Imap => Profile::MailConnectorProfileImap as i32,
        MailConnectorProfileV1::ImapSmtp => Profile::MailConnectorProfileImapSmtp as i32,
        MailConnectorProfileV1::Gmail => Profile::MailConnectorProfileGmail as i32,
    }
}

fn connector_profile_from_wire(
    profile: i32,
) -> Result<MailConnectorProfileV1, MailClientWireErrorV1> {
    use wire::MailConnectorProfileV1 as Profile;
    match Profile::try_from(profile).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Profile::MailConnectorProfileImap => Ok(MailConnectorProfileV1::Imap),
        Profile::MailConnectorProfileImapSmtp => Ok(MailConnectorProfileV1::ImapSmtp),
        Profile::MailConnectorProfileGmail => Ok(MailConnectorProfileV1::Gmail),
        Profile::MailConnectorProfileUnspecified => Err(MailClientWireErrorV1::InvalidPayload),
    }
}

fn provider_path_readiness_to_wire(readiness: MailProviderPathReadinessV1) -> i32 {
    use wire::MailProviderPathReadinessV1 as Readiness;
    match readiness {
        MailProviderPathReadinessV1::NotConfigured => {
            Readiness::MailProviderPathReadinessNotConfigured as i32
        }
        MailProviderPathReadinessV1::CredentialRequired => {
            Readiness::MailProviderPathReadinessCredentialRequired as i32
        }
        MailProviderPathReadinessV1::PendingRestart => {
            Readiness::MailProviderPathReadinessPendingRestart as i32
        }
        MailProviderPathReadinessV1::Ready => Readiness::MailProviderPathReadinessReady as i32,
        MailProviderPathReadinessV1::Retired => Readiness::MailProviderPathReadinessRetired as i32,
        MailProviderPathReadinessV1::Deleted => Readiness::MailProviderPathReadinessDeleted as i32,
        MailProviderPathReadinessV1::Degraded => {
            Readiness::MailProviderPathReadinessDegraded as i32
        }
    }
}

fn provider_path_readiness_from_wire(
    readiness: i32,
) -> Result<MailProviderPathReadinessV1, MailClientWireErrorV1> {
    use wire::MailProviderPathReadinessV1 as Readiness;
    match Readiness::try_from(readiness).map_err(|_| MailClientWireErrorV1::InvalidPayload)? {
        Readiness::MailProviderPathReadinessNotConfigured => {
            Ok(MailProviderPathReadinessV1::NotConfigured)
        }
        Readiness::MailProviderPathReadinessCredentialRequired => {
            Ok(MailProviderPathReadinessV1::CredentialRequired)
        }
        Readiness::MailProviderPathReadinessPendingRestart => {
            Ok(MailProviderPathReadinessV1::PendingRestart)
        }
        Readiness::MailProviderPathReadinessReady => Ok(MailProviderPathReadinessV1::Ready),
        Readiness::MailProviderPathReadinessRetired => Ok(MailProviderPathReadinessV1::Retired),
        Readiness::MailProviderPathReadinessDeleted => Ok(MailProviderPathReadinessV1::Deleted),
        Readiness::MailProviderPathReadinessDegraded => Ok(MailProviderPathReadinessV1::Degraded),
        Readiness::MailProviderPathReadinessUnspecified => {
            Err(MailClientWireErrorV1::InvalidPayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account_status(connection_id: &str) -> MailAccountStatusV1 {
        MailAccountStatusV1 {
            connection_id: connection_id.to_owned(),
            configuration_instance_id: format!("{connection_id}-configuration"),
            settings_revision: 4,
            runtime_generation: 9,
            readiness: MailAccountReadinessV1::PendingRestart,
            connector_profile: MailConnectorProfileV1::ImapSmtp,
            sync_readiness: MailProviderPathReadinessV1::Ready,
            delivery_readiness: MailProviderPathReadinessV1::PendingRestart,
            bindings: vec![
                MailCredentialBindingStatusV1 {
                    purpose: MailCredentialPurposeV1::ImapPassword,
                    state: MailCredentialBindingStateV1::Active,
                    binding_revision: Some(1),
                    credential_revision: Some(2),
                    applied_runtime_generation: Some(9),
                },
                MailCredentialBindingStatusV1 {
                    purpose: MailCredentialPurposeV1::SmtpPassword,
                    state: MailCredentialBindingStateV1::PendingRestart,
                    binding_revision: Some(3),
                    credential_revision: Some(4),
                    applied_runtime_generation: None,
                },
            ],
            lifecycle_revision: 3,
            lifecycle_operation_id: Some(format!("{connection_id}-lifecycle-3")),
        }
    }

    #[test]
    fn account_binding_round_trips_without_secret_or_record_identifiers() {
        let request = MailBindCredentialRequestV1 {
            connection_id: "mail-account".to_owned(),
            purpose: MailCredentialPurposeV1::SmtpPassword,
            expected_binding_revision: 2,
            credential_revision: 3,
        };
        assert_eq!(
            decode_bind_request(&encode_bind_request(&request).expect("encode")),
            Ok(request)
        );
    }

    #[test]
    fn sanitized_account_status_round_trips_connector_and_path_readiness() {
        let status = sample_account_status("mail-account");

        assert_eq!(
            decode_account_status(&encode_account_status(&status).expect("encode")),
            Ok(status)
        );
    }

    #[test]
    fn account_catalog_is_sorted_bounded_and_round_trips_canonically() {
        let catalog = MailAccountCatalogV1 {
            accounts: vec![
                sample_account_status("mail-account-a"),
                sample_account_status("mail-account-b"),
            ],
        };
        assert_eq!(
            decode_account_catalog(&encode_account_catalog(&catalog).expect("encode catalog")),
            Ok(catalog.clone())
        );

        let mut unsorted = catalog;
        unsorted.accounts.reverse();
        assert_eq!(
            encode_account_catalog(&unsorted),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
        assert_eq!(
            encode_account_catalog(&MailAccountCatalogV1 {
                accounts: Vec::new(),
            }),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
