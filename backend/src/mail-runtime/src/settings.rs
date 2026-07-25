//! Mail-owned decoding of one admitted generic settings snapshot.

use hermes_mail_api::{
    GmailApiEndpointV1, MailAccountConfigurationV1, MailGmailConfigurationV1,
    MailImapConfigurationV1, MailInboundTransportV1, SmtpEndpointV1, valid_account_configuration,
};
use hermes_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

use crate::MailCredentialRevisionsV1;

const CONNECTION_ID: &str = "mail.connection_id";
const IMAP_HOST: &str = "mail.imap.host";
const IMAP_PORT: &str = "mail.imap.port";
const IMAP_USERNAME: &str = "mail.imap.username";
const SYNC_WINDOW: &str = "mail.sync.window";
const SYNC_WINDOWS: &str = "mail.sync.windows";
const IMAP_PASSWORD_REVISION: &str = "mail.imap.password_revision";
const SMTP_ENABLED: &str = "mail.smtp.enabled";
const SMTP_CA_CERTIFICATE_PEM: &str = "mail.smtp.ca_certificate_pem";
const SMTP_HOST: &str = "mail.smtp.host";
const SMTP_PORT: &str = "mail.smtp.port";
const SMTP_USERNAME: &str = "mail.smtp.username";
const SMTP_FROM_ADDRESS: &str = "mail.smtp.from_address";
const SMTP_PASSWORD_REVISION: &str = "mail.smtp.password_revision";
const INBOUND_KIND: &str = "mail.inbound.kind";
const GMAIL_API_HOST: &str = "mail.gmail.api_host";
const GMAIL_API_PORT: &str = "mail.gmail.api_port";
const GMAIL_CA_CERTIFICATE_PEM: &str = "mail.gmail.ca_certificate_pem";
const GMAIL_USER_ID: &str = "mail.gmail.user_id";
const GMAIL_FROM_ADDRESS: &str = "mail.gmail.from_address";
const GMAIL_ACCESS_TOKEN_REVISION: &str = "mail.gmail.access_token_revision";

pub const MAIL_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const MAIL_SETTINGS_SCHEMA_REVISION_V1: u32 = 3;

/// The Mail integration owns these configuration-instance settings. They are
/// deliberately hidden from generic client reads: endpoint details and
/// credential revisions are runtime configuration, not Communications state.
#[must_use]
pub fn mail_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: MAIL_SETTINGS_SCHEMA_MAJOR_V1,
        revision: MAIL_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(CONNECTION_ID, SettingValueTypeV1::String, "Connection ID"),
            definition(
                GMAIL_ACCESS_TOKEN_REVISION,
                SettingValueTypeV1::UnsignedInteger,
                "Gmail access-token revision",
            ),
            definition(GMAIL_API_HOST, SettingValueTypeV1::String, "Gmail API host"),
            definition(
                GMAIL_API_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Gmail API port",
            ),
            definition(
                GMAIL_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "Gmail CA certificate",
            ),
            definition(
                GMAIL_FROM_ADDRESS,
                SettingValueTypeV1::String,
                "Gmail from address",
            ),
            definition(GMAIL_USER_ID, SettingValueTypeV1::String, "Gmail user ID"),
            definition(IMAP_HOST, SettingValueTypeV1::String, "IMAP host"),
            definition(
                IMAP_PASSWORD_REVISION,
                SettingValueTypeV1::UnsignedInteger,
                "IMAP password revision",
            ),
            definition(IMAP_PORT, SettingValueTypeV1::UnsignedInteger, "IMAP port"),
            definition(IMAP_USERNAME, SettingValueTypeV1::String, "IMAP username"),
            definition(
                INBOUND_KIND,
                SettingValueTypeV1::String,
                "Inbound transport",
            ),
            definition(
                SMTP_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "SMTP CA certificate",
            ),
            definition(SMTP_ENABLED, SettingValueTypeV1::Boolean, "SMTP enabled"),
            definition(
                SMTP_FROM_ADDRESS,
                SettingValueTypeV1::String,
                "SMTP from address",
            ),
            definition(SMTP_HOST, SettingValueTypeV1::String, "SMTP host"),
            definition(
                SMTP_PASSWORD_REVISION,
                SettingValueTypeV1::UnsignedInteger,
                "SMTP password revision",
            ),
            definition(SMTP_PORT, SettingValueTypeV1::UnsignedInteger, "SMTP port"),
            definition(SMTP_USERNAME, SettingValueTypeV1::String, "SMTP username"),
            definition(
                SYNC_WINDOW,
                SettingValueTypeV1::UnsignedInteger,
                "Sync window",
            ),
            definition(
                SYNC_WINDOWS,
                SettingValueTypeV1::UnsignedInteger,
                "Sync windows",
            ),
        ],
    }
}

#[must_use]
pub fn mail_settings_schema_bytes_v1() -> Vec<u8> {
    mail_settings_schema_v1().encode_to_vec()
}

fn definition(
    setting_id: &str,
    value_type: SettingValueTypeV1,
    display_name: &str,
) -> SettingDefinitionV1 {
    SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: value_type as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Hidden as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: display_name.to_owned(),
    }
}

pub struct MailRuntimeSettingsV1 {
    pub account: MailAccountConfigurationV1,
    pub credential_revisions: MailCredentialRevisionsV1,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<MailRuntimeSettingsV1, String> {
    let inbound = match required_string(snapshot, INBOUND_KIND)?.as_str() {
        "imap" => {
            for setting_id in [
                GMAIL_API_HOST,
                GMAIL_API_PORT,
                GMAIL_CA_CERTIFICATE_PEM,
                GMAIL_FROM_ADDRESS,
                GMAIL_USER_ID,
            ] {
                absent(snapshot, setting_id)?;
            }
            MailInboundTransportV1::Imap(MailImapConfigurationV1 {
                host: required_string(snapshot, IMAP_HOST)?,
                port: u16::try_from(required_unsigned(snapshot, IMAP_PORT)?)
                    .map_err(|_| invalid_settings())?,
                username: required_string(snapshot, IMAP_USERNAME)?,
            })
        }
        "gmail" => MailInboundTransportV1::Gmail(MailGmailConfigurationV1 {
            user_id: required_string(snapshot, GMAIL_USER_ID)?,
            from_address: required_string(snapshot, GMAIL_FROM_ADDRESS)?,
            api_endpoint: GmailApiEndpointV1 {
                host: required_string(snapshot, GMAIL_API_HOST)?,
                port: u16::try_from(required_unsigned(snapshot, GMAIL_API_PORT)?)
                    .map_err(|_| invalid_settings())?,
                ca_certificate_pem: optional_string(snapshot, GMAIL_CA_CERTIFICATE_PEM)?,
            },
        }),
        _ => return Err(invalid_settings()),
    };
    let account = MailAccountConfigurationV1 {
        connection_id: required_string(snapshot, CONNECTION_ID)?,
        inbound,
        sync_window: u32::try_from(required_unsigned(snapshot, SYNC_WINDOW)?)
            .map_err(|_| invalid_settings())?,
        sync_windows: u32::try_from(required_unsigned(snapshot, SYNC_WINDOWS)?)
            .map_err(|_| invalid_settings())?,
        smtp_endpoint: smtp_endpoint(snapshot)?,
    };
    if !valid_account_configuration(&account) {
        return Err(invalid_settings());
    }
    let (imap_password, gmail_access_token) = match &account.inbound {
        MailInboundTransportV1::Imap(_) => {
            let revision = required_unsigned(snapshot, IMAP_PASSWORD_REVISION)?;
            if revision == 0 {
                return Err(invalid_settings());
            }
            absent(snapshot, GMAIL_ACCESS_TOKEN_REVISION)?;
            (Some(revision), None)
        }
        MailInboundTransportV1::Gmail(_) => {
            for setting_id in [IMAP_HOST, IMAP_PORT, IMAP_USERNAME] {
                absent(snapshot, setting_id)?;
            }
            let revision = required_unsigned(snapshot, GMAIL_ACCESS_TOKEN_REVISION)?;
            if revision == 0 {
                return Err(invalid_settings());
            }
            absent(snapshot, IMAP_PASSWORD_REVISION)?;
            (None, Some(revision))
        }
    };
    let smtp_password = if account.smtp_endpoint.is_some() {
        let revision = required_unsigned(snapshot, SMTP_PASSWORD_REVISION)?;
        Some(
            (revision != 0)
                .then_some(revision)
                .ok_or_else(invalid_settings)?,
        )
    } else {
        absent(snapshot, SMTP_PASSWORD_REVISION)?;
        None
    };
    if matches!(account.inbound, MailInboundTransportV1::Gmail(_)) && smtp_password.is_some() {
        return Err(invalid_settings());
    }
    Ok(MailRuntimeSettingsV1 {
        account,
        credential_revisions: MailCredentialRevisionsV1 {
            imap_password,
            gmail_access_token,
            smtp_password,
        },
    })
}

fn smtp_endpoint(snapshot: &SettingsSnapshotV1) -> Result<Option<SmtpEndpointV1>, String> {
    if !required_boolean(snapshot, SMTP_ENABLED)? {
        for setting_id in [
            SMTP_CA_CERTIFICATE_PEM,
            SMTP_HOST,
            SMTP_PORT,
            SMTP_USERNAME,
            SMTP_FROM_ADDRESS,
        ] {
            absent(snapshot, setting_id)?;
        }
        return Ok(None);
    }
    Ok(Some(SmtpEndpointV1 {
        host: required_string(snapshot, SMTP_HOST)?,
        port: u16::try_from(required_unsigned(snapshot, SMTP_PORT)?)
            .map_err(|_| invalid_settings())?,
        username: required_string(snapshot, SMTP_USERNAME)?,
        from_address: required_string(snapshot, SMTP_FROM_ADDRESS)?,
        ca_certificate_pem: optional_string(snapshot, SMTP_CA_CERTIFICATE_PEM)?,
    }))
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(invalid_settings()),
    }
}

fn required_unsigned(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<u64, String> {
    match value(snapshot, setting_id)? {
        Value::UnsignedIntegerValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn optional_string(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<Option<String>, String> {
    let entries = snapshot
        .values
        .iter()
        .filter(|entry| entry.setting_id == setting_id)
        .collect::<Vec<_>>();
    match entries.as_slice() {
        [] => Ok(None),
        [entry] => match entry.value.as_ref().and_then(|value| value.value.as_ref()) {
            Some(Value::StringValue(value)) if !value.trim().is_empty() => Ok(Some(value.clone())),
            _ => Err(invalid_settings()),
        },
        _ => Err(invalid_settings()),
    }
}

fn required_boolean(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<bool, String> {
    match value(snapshot, setting_id)? {
        Value::BooleanValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn absent(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<(), String> {
    (!snapshot
        .values
        .iter()
        .any(|entry| entry.setting_id == setting_id))
    .then_some(())
    .ok_or_else(invalid_settings)
}

fn value<'a>(snapshot: &'a SettingsSnapshotV1, setting_id: &str) -> Result<&'a Value, String> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(invalid_settings());
            }
        }
    }
    selected.flatten().ok_or_else(invalid_settings)
}

fn invalid_settings() -> String {
    "Mail runtime settings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::validation::descriptor::validate_settings_schema_v1;

    use super::*;

    #[test]
    fn schema_is_versioned_hidden_and_configuration_scoped() {
        let schema = mail_settings_schema_v1();

        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert!(schema.definitions.iter().all(|definition| {
            definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
                && definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.client_visibility == SettingClientVisibilityV1::Hidden as i32
                && definition.fresh_owner_proof_required
        }));
        assert_eq!(schema.definitions.len(), 21);
    }

    #[test]
    fn production_gmail_endpoint_defaults_are_canonical() {
        assert_eq!(hermes_mail_api::GMAIL_API_HOST, "gmail.googleapis.com");
        assert_eq!(hermes_mail_api::GMAIL_API_HTTPS_PORT, 443);
    }
}
