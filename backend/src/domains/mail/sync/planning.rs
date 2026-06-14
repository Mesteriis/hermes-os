use serde_json::Value;

use crate::domains::mail::core::{
    EmailProviderKind, ProviderAccount, ProviderAccountSecretPurpose,
};

use super::errors::EmailSyncPlanError;
use super::models::{EmailSyncAdapterConfig, EmailSyncPlan};
use super::validation::{
    optional_string, reject_secret_like_config_keys, required_bool, required_port, required_string,
    validate_no_control_chars, validate_non_empty,
};

pub fn plan_email_sync(account: &ProviderAccount) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let account_id = validate_non_empty("account_id", &account.account_id)?;
    reject_secret_like_config_keys(&account.config)?;

    match account.provider_kind {
        EmailProviderKind::Gmail => plan_gmail_sync(account, account_id),
        EmailProviderKind::Icloud | EmailProviderKind::Imap => plan_imap_sync(account, account_id),
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb => Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "provider_kind",
            message: "email sync supports only gmail, icloud or imap",
        }),
    }
}

pub fn imap_mailbox_stream_id(mailbox: &str) -> String {
    let mut stream_id = String::from("imap:");

    for character in mailbox.chars() {
        match character {
            '%' => stream_id.push_str("%25"),
            ':' => stream_id.push_str("%3A"),
            _ => stream_id.push(character),
        }
    }

    stream_id
}

fn plan_gmail_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let history_stream_id = optional_string(&account.config, "history_stream_id")?
        .unwrap_or_else(|| "gmail:history".to_owned());
    validate_non_empty("history_stream_id", &history_stream_id)?;
    validate_no_control_chars("history_stream_id", &history_stream_id)?;

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::OauthToken,
        stream_id: history_stream_id.clone(),
        adapter_config: EmailSyncAdapterConfig::Gmail { history_stream_id },
    })
}

fn plan_imap_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let host = required_string(&account.config, "host")?;
    let port = required_port(&account.config, "port")?;
    let tls = required_bool(&account.config, "tls")?;
    let mailbox =
        optional_string(&account.config, "mailbox")?.unwrap_or_else(|| "INBOX".to_owned());
    validate_non_empty("mailbox", &mailbox)?;
    validate_no_control_chars("mailbox", &mailbox)?;
    let stream_id = imap_mailbox_stream_id(&mailbox);

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::ImapPassword,
        stream_id,
        adapter_config: EmailSyncAdapterConfig::Imap {
            host,
            port,
            tls,
            mailbox,
        },
    })
}
