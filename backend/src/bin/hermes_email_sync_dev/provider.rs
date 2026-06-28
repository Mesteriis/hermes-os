use hermes_hub_backend::domains::communications::core::EmailProviderKind;

use crate::errors::DevEmailSyncError;

pub(super) const DEFAULT_IMAP_PORT: u16 = 993;

const DEFAULT_ICLOUD_IMAP_HOST: &str = "imap.mail.me.com";

pub(super) fn parse_provider_kind(value: &str) -> Result<EmailProviderKind, DevEmailSyncError> {
    let provider_kind = EmailProviderKind::try_from(value.trim())
        .map_err(|_| DevEmailSyncError::InvalidProviderKind(value.to_owned()))?;
    match provider_kind {
        EmailProviderKind::Icloud | EmailProviderKind::Imap => Ok(provider_kind),
        EmailProviderKind::Gmail
        | EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer => {
            Err(DevEmailSyncError::UnsupportedProviderForDevSync)
        }
    }
}

pub(super) fn default_host(provider_kind: EmailProviderKind) -> &'static str {
    match provider_kind {
        EmailProviderKind::Icloud => DEFAULT_ICLOUD_IMAP_HOST,
        EmailProviderKind::Imap => "localhost",
        EmailProviderKind::Gmail
        | EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer => "",
    }
}
