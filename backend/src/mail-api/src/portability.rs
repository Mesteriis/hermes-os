//! Mail-owned validation for the non-secret portability contract.

use crate::{
    GOOGLE_PEOPLE_API_HOST_V1, GOOGLE_PEOPLE_API_PORT_V1, GmailApiEndpointV1,
    GmailOAuthConfigurationV1, GmailOAuthEndpointV1, ICLOUD_CARDDAV_BASE_PATH_V1,
    ICLOUD_CARDDAV_HOST_V1, ICLOUD_CARDDAV_PORT_V1,
    MailAccountConfigurationV1 as RuntimeMailAccountConfigurationV1,
    MailAddressBookConfigurationV1, MailAddressBookProviderV1, MailAddressBookTlsEndpointV1,
    MailCardDavEndpointV1, MailGmailConfigurationV1, MailImapConfigurationV1,
    MailInboundTransportV1, SmtpEndpointV1, portability_wire_generated as wire,
    valid_account_configuration, valid_address_book_configuration, valid_gmail_oauth_configuration,
};

pub const MAIL_ACCOUNT_EXPORT_MAJOR_V1: u32 = 1;
pub const MAIL_SETTINGS_SCHEMA_MAJOR_V2: u32 = 2;
pub const MAIL_SETTINGS_SCHEMA_REVISION_V2: u32 = 4;
const MAX_REGISTRATION_ID_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAccountExportValidationErrorV1 {
    Invalid,
}

pub fn validate_mail_account_export_v1(
    export: &wire::MailAccountExportV1,
) -> Result<(), MailAccountExportValidationErrorV1> {
    if export.major != MAIL_ACCOUNT_EXPORT_MAJOR_V1
        || export.exported_at_unix_millis == 0
        || !valid_registration_id(&export.source_registration_id)
        || export.settings_schema_major != MAIL_SETTINGS_SCHEMA_MAJOR_V2
        || export.settings_schema_revision != MAIL_SETTINGS_SCHEMA_REVISION_V2
        || export.effective_settings_revision == 0
    {
        return Err(MailAccountExportValidationErrorV1::Invalid);
    }
    let configuration = export
        .configuration
        .as_ref()
        .ok_or(MailAccountExportValidationErrorV1::Invalid)?;
    let account = runtime_configuration(configuration)?;
    let address_book = address_book_configuration(configuration)?;
    if !valid_account_configuration(&account)
        || !valid_address_book_configuration(&address_book, &account.inbound)
        || !profile_matches(export.connector_profile, configuration)
        || !valid_readiness(export.readiness)
        || !valid_path_readiness(export.sync_readiness)
        || !valid_path_readiness(export.delivery_readiness)
    {
        return Err(MailAccountExportValidationErrorV1::Invalid);
    }
    Ok(())
}

fn address_book_configuration(
    configuration: &wire::MailAccountConfigurationV1,
) -> Result<MailAddressBookConfigurationV1, MailAccountExportValidationErrorV1> {
    let provider =
        match wire::MailAddressBookProviderV1::try_from(configuration.address_book_provider).ok() {
            Some(wire::MailAddressBookProviderV1::MailAddressBookProviderNone) => {
                MailAddressBookProviderV1::None
            }
            Some(wire::MailAddressBookProviderV1::MailAddressBookProviderGooglePeople) => {
                MailAddressBookProviderV1::GooglePeople
            }
            Some(wire::MailAddressBookProviderV1::MailAddressBookProviderIcloudCardDav) => {
                MailAddressBookProviderV1::IcloudCardDav
            }
            _ => return Err(MailAccountExportValidationErrorV1::Invalid),
        };
    Ok(MailAddressBookConfigurationV1 {
        provider,
        carddav_username: configuration.carddav_username.clone(),
        google_people_endpoint: (provider == MailAddressBookProviderV1::GooglePeople).then(|| {
            MailAddressBookTlsEndpointV1 {
                host: GOOGLE_PEOPLE_API_HOST_V1.to_owned(),
                port: GOOGLE_PEOPLE_API_PORT_V1,
                ca_certificate_pem: None,
            }
        }),
        carddav_endpoint: (provider == MailAddressBookProviderV1::IcloudCardDav).then(|| {
            MailCardDavEndpointV1 {
                tls: MailAddressBookTlsEndpointV1 {
                    host: ICLOUD_CARDDAV_HOST_V1.to_owned(),
                    port: ICLOUD_CARDDAV_PORT_V1,
                    ca_certificate_pem: None,
                },
                base_path: ICLOUD_CARDDAV_BASE_PATH_V1.to_owned(),
            }
        }),
    })
}

fn runtime_configuration(
    configuration: &wire::MailAccountConfigurationV1,
) -> Result<RuntimeMailAccountConfigurationV1, MailAccountExportValidationErrorV1> {
    let inbound = match configuration.inbound.as_ref() {
        Some(wire::mail_account_configuration_v1::Inbound::Imap(imap)) => {
            MailInboundTransportV1::Imap(MailImapConfigurationV1 {
                host: imap.host.clone(),
                port: u16::try_from(imap.port)
                    .map_err(|_| MailAccountExportValidationErrorV1::Invalid)?,
                username: imap.username.clone(),
            })
        }
        Some(wire::mail_account_configuration_v1::Inbound::Gmail(gmail)) => {
            if gmail.from_address.is_empty() {
                return Err(MailAccountExportValidationErrorV1::Invalid);
            }
            let api_endpoint = gmail
                .api_endpoint
                .as_ref()
                .ok_or(MailAccountExportValidationErrorV1::Invalid)?;
            let oauth = gmail_oauth_configuration(gmail)?;
            if !valid_gmail_oauth_configuration(&oauth) {
                return Err(MailAccountExportValidationErrorV1::Invalid);
            }
            MailInboundTransportV1::Gmail(MailGmailConfigurationV1 {
                user_id: gmail.user_id.clone(),
                from_address: Some(gmail.from_address.clone()),
                api_endpoint: GmailApiEndpointV1 {
                    host: api_endpoint.host.clone(),
                    port: u16::try_from(api_endpoint.port)
                        .map_err(|_| MailAccountExportValidationErrorV1::Invalid)?,
                    ca_certificate_pem: api_endpoint.ca_certificate_pem.clone(),
                },
            })
        }
        None => return Err(MailAccountExportValidationErrorV1::Invalid),
    };
    let smtp_endpoint = configuration
        .smtp
        .as_ref()
        .map(|smtp| {
            Ok(SmtpEndpointV1 {
                host: smtp.host.clone(),
                port: u16::try_from(smtp.port)
                    .map_err(|_| MailAccountExportValidationErrorV1::Invalid)?,
                username: smtp.username.clone(),
                from_address: smtp.from_address.clone(),
                ca_certificate_pem: smtp.ca_certificate_pem.clone(),
            })
        })
        .transpose()?;
    Ok(RuntimeMailAccountConfigurationV1 {
        connection_id: configuration.connection_id.clone(),
        inbound,
        sync_window: configuration.sync_window,
        sync_windows: configuration.sync_windows,
        smtp_endpoint,
    })
}

fn gmail_oauth_configuration(
    gmail: &wire::MailGmailConfigurationV1,
) -> Result<GmailOAuthConfigurationV1, MailAccountExportValidationErrorV1> {
    Ok(GmailOAuthConfigurationV1 {
        client_id: gmail.oauth_client_id.clone(),
        redirect_uri: gmail.oauth_redirect_uri.clone(),
        authorization_endpoint: oauth_endpoint(gmail.oauth_authorization_endpoint.as_ref())?,
        token_endpoint: oauth_endpoint(gmail.oauth_token_endpoint.as_ref())?,
    })
}

fn oauth_endpoint(
    endpoint: Option<&wire::MailHttpEndpointV1>,
) -> Result<GmailOAuthEndpointV1, MailAccountExportValidationErrorV1> {
    let endpoint = endpoint.ok_or(MailAccountExportValidationErrorV1::Invalid)?;
    Ok(GmailOAuthEndpointV1 {
        host: endpoint.host.clone(),
        port: u16::try_from(endpoint.port)
            .map_err(|_| MailAccountExportValidationErrorV1::Invalid)?,
        path: endpoint.path.clone(),
        ca_certificate_pem: endpoint.ca_certificate_pem.clone(),
    })
}

fn profile_matches(profile: i32, configuration: &wire::MailAccountConfigurationV1) -> bool {
    matches!(
        (
            wire::MailExportConnectorProfileV1::try_from(profile).ok(),
            configuration.inbound.as_ref(),
            configuration.smtp.is_some(),
        ),
        (
            Some(wire::MailExportConnectorProfileV1::MailExportConnectorProfileImap),
            Some(wire::mail_account_configuration_v1::Inbound::Imap(_)),
            false,
        ) | (
            Some(wire::MailExportConnectorProfileV1::MailExportConnectorProfileImapSmtp),
            Some(wire::mail_account_configuration_v1::Inbound::Imap(_)),
            true,
        ) | (
            Some(wire::MailExportConnectorProfileV1::MailExportConnectorProfileGmail),
            Some(wire::mail_account_configuration_v1::Inbound::Gmail(_)),
            false,
        )
    )
}

fn valid_readiness(value: i32) -> bool {
    wire::MailExportAccountReadinessV1::try_from(value).is_ok_and(|value| {
        value != wire::MailExportAccountReadinessV1::MailExportAccountReadinessUnspecified
    })
}

fn valid_path_readiness(value: i32) -> bool {
    wire::MailExportProviderPathReadinessV1::try_from(value).is_ok_and(|value| {
        value != wire::MailExportProviderPathReadinessV1::MailExportProviderPathReadinessUnspecified
    })
}

fn valid_registration_id(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_REGISTRATION_ID_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_export_is_non_secret_and_profile_consistent() {
        let export = imap_export();
        assert_eq!(validate_mail_account_export_v1(&export), Ok(()));

        let mut mismatched = export.clone();
        mismatched.connector_profile =
            wire::MailExportConnectorProfileV1::MailExportConnectorProfileGmail as i32;
        assert_eq!(
            validate_mail_account_export_v1(&mismatched),
            Err(MailAccountExportValidationErrorV1::Invalid)
        );

        let mut unsupported_schema = export;
        unsupported_schema.settings_schema_revision += 1;
        assert_eq!(
            validate_mail_account_export_v1(&unsupported_schema),
            Err(MailAccountExportValidationErrorV1::Invalid)
        );
    }

    fn imap_export() -> wire::MailAccountExportV1 {
        wire::MailAccountExportV1 {
            major: MAIL_ACCOUNT_EXPORT_MAJOR_V1,
            exported_at_unix_millis: 1,
            source_registration_id: "mail-registration".to_owned(),
            settings_schema_major: MAIL_SETTINGS_SCHEMA_MAJOR_V2,
            settings_schema_revision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
            effective_settings_revision: 1,
            connector_profile:
                wire::MailExportConnectorProfileV1::MailExportConnectorProfileImapSmtp as i32,
            readiness: wire::MailExportAccountReadinessV1::MailExportAccountReadinessReady as i32,
            sync_readiness:
                wire::MailExportProviderPathReadinessV1::MailExportProviderPathReadinessReady as i32,
            delivery_readiness:
                wire::MailExportProviderPathReadinessV1::MailExportProviderPathReadinessReady as i32,
            configuration: Some(wire::MailAccountConfigurationV1 {
                connection_id: "mail-account".to_owned(),
                sync_window: 100,
                sync_windows: 2,
                inbound: Some(wire::mail_account_configuration_v1::Inbound::Imap(
                    wire::MailImapConfigurationV1 {
                        host: "imap.example.test".to_owned(),
                        port: 993,
                        username: "owner@example.test".to_owned(),
                    },
                )),
                smtp: Some(wire::MailSmtpConfigurationV1 {
                    host: "smtp.example.test".to_owned(),
                    port: 465,
                    username: "owner@example.test".to_owned(),
                    from_address: "owner@example.test".to_owned(),
                    ca_certificate_pem: None,
                }),
                address_book_provider: wire::MailAddressBookProviderV1::MailAddressBookProviderNone
                    as i32,
                carddav_username: None,
            }),
        }
    }
}
