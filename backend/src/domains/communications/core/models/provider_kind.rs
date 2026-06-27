use serde::{Deserialize, Serialize};

use super::super::errors::CommunicationIngestionError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationProviderKind {
    Gmail,
    Icloud,
    Imap,
    TelegramUser,
    TelegramBot,
    WhatsappWeb,
    WhatsappBusinessCloud,
}

pub type EmailProviderKind = CommunicationProviderKind;

impl CommunicationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::Imap => "imap",
            Self::TelegramUser => "telegram_user",
            Self::TelegramBot => "telegram_bot",
            Self::WhatsappWeb => "whatsapp_web",
            Self::WhatsappBusinessCloud => "whatsapp_business_cloud",
        }
    }

    pub fn is_email(self) -> bool {
        matches!(self, Self::Gmail | Self::Icloud | Self::Imap)
    }

    pub fn is_telegram(self) -> bool {
        matches!(self, Self::TelegramUser | Self::TelegramBot)
    }

    pub fn is_whatsapp(self) -> bool {
        matches!(self, Self::WhatsappWeb | Self::WhatsappBusinessCloud)
    }
}

impl TryFrom<&str> for CommunicationProviderKind {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            "whatsapp_business_cloud" => Ok(Self::WhatsappBusinessCloud),
            other => Err(CommunicationIngestionError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}
