//! Mail-owned decoding of one admitted generic settings snapshot.

use hermes_mail_api::{MailAccountConfigurationV1, MailGmailConfigurationV1, MailImapConfigurationV1, MailInboundTransportV1, SmtpEndpointV1, valid_account_configuration};
use hermes_runtime_protocol::v1::{SettingsSnapshotV1, setting_value_v1::Value};

use crate::MailCredentialRevisionsV1;

const CONNECTION_ID: &str = "mail.connection_id";
const IMAP_HOST: &str = "mail.imap.host";
const IMAP_PORT: &str = "mail.imap.port";
const IMAP_USERNAME: &str = "mail.imap.username";
const SYNC_WINDOW: &str = "mail.sync.window";
const SYNC_WINDOWS: &str = "mail.sync.windows";
const IMAP_PASSWORD_REVISION: &str = "mail.imap.password_revision";
const SMTP_ENABLED: &str = "mail.smtp.enabled";
const SMTP_HOST: &str = "mail.smtp.host";
const SMTP_PORT: &str = "mail.smtp.port";
const SMTP_USERNAME: &str = "mail.smtp.username";
const SMTP_FROM_ADDRESS: &str = "mail.smtp.from_address";
const SMTP_PASSWORD_REVISION: &str = "mail.smtp.password_revision";
const INBOUND_KIND: &str = "mail.inbound.kind";
const GMAIL_USER_ID: &str = "mail.gmail.user_id";
const GMAIL_FROM_ADDRESS: &str = "mail.gmail.from_address";
const GMAIL_ACCESS_TOKEN_REVISION: &str = "mail.gmail.access_token_revision";

pub struct MailRuntimeSettingsV1 {
    pub account: MailAccountConfigurationV1,
    pub credential_revisions: MailCredentialRevisionsV1,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<MailRuntimeSettingsV1, String> {
    let inbound = match required_string(snapshot, INBOUND_KIND)?.as_str() {
        "imap" => MailInboundTransportV1::Imap(MailImapConfigurationV1 {
            host: required_string(snapshot, IMAP_HOST)?,
            port: u16::try_from(required_unsigned(snapshot, IMAP_PORT)?)
                .map_err(|_| invalid_settings())?,
            username: required_string(snapshot, IMAP_USERNAME)?,
        }),
        "gmail" => MailInboundTransportV1::Gmail(MailGmailConfigurationV1 {
            user_id: required_string(snapshot, GMAIL_USER_ID)?,
            from_address: required_string(snapshot, GMAIL_FROM_ADDRESS)?,
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
            if revision == 0 { return Err(invalid_settings()); }
            absent(snapshot, GMAIL_ACCESS_TOKEN_REVISION)?;
            (Some(revision), None)
        }
        MailInboundTransportV1::Gmail(_) => {
            let revision = required_unsigned(snapshot, GMAIL_ACCESS_TOKEN_REVISION)?;
            if revision == 0 { return Err(invalid_settings()); }
            absent(snapshot, IMAP_PASSWORD_REVISION)?;
            (None, Some(revision))
        }
    };
    let smtp_password = if account.smtp_endpoint.is_some() {
        let revision = required_unsigned(snapshot, SMTP_PASSWORD_REVISION)?;
        Some((revision != 0).then_some(revision).ok_or_else(invalid_settings)?)
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
        for setting_id in [SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_FROM_ADDRESS] {
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

fn required_boolean(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<bool, String> {
    match value(snapshot, setting_id)? {
        Value::BooleanValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn absent(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<(), String> {
    (!snapshot.values.iter().any(|entry| entry.setting_id == setting_id))
        .then_some(())
        .ok_or_else(invalid_settings)
}

fn value<'a>(
    snapshot: &'a SettingsSnapshotV1,
    setting_id: &str,
) -> Result<&'a Value, String> {
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
