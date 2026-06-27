use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

use super::super::errors::CommunicationIngestionError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAccountSecretPurpose {
    OauthToken,
    ImapPassword,
    SmtpPassword,
    TelegramApiHash,
    TelegramSessionKey,
    TelegramBotToken,
    WhatsappWebSessionKey,
    WhatsappBusinessCloudAccessToken,
    WhatsappBusinessCloudAppSecret,
    WhatsappBusinessCloudWebhookVerifyToken,
}

impl ProviderAccountSecretPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::ImapPassword => "imap_password",
            Self::SmtpPassword => "smtp_password",
            Self::TelegramApiHash => "telegram_api_hash",
            Self::TelegramSessionKey => "telegram_session_key",
            Self::TelegramBotToken => "telegram_bot_token",
            Self::WhatsappWebSessionKey => "whatsapp_web_session_key",
            Self::WhatsappBusinessCloudAccessToken => "whatsapp_business_cloud_access_token",
            Self::WhatsappBusinessCloudAppSecret => "whatsapp_business_cloud_app_secret",
            Self::WhatsappBusinessCloudWebhookVerifyToken => {
                "whatsapp_business_cloud_webhook_verify_token"
            }
        }
    }

    pub fn accepts_secret_kind(self, secret_kind: SecretKind) -> bool {
        match self {
            Self::OauthToken => secret_kind == SecretKind::OauthToken,
            Self::ImapPassword | Self::SmtpPassword => {
                matches!(secret_kind, SecretKind::AppPassword | SecretKind::Password)
            }
            Self::TelegramApiHash | Self::TelegramBotToken => secret_kind == SecretKind::ApiToken,
            Self::TelegramSessionKey | Self::WhatsappWebSessionKey => {
                matches!(secret_kind, SecretKind::PrivateKey | SecretKind::Other)
            }
            Self::WhatsappBusinessCloudAccessToken
            | Self::WhatsappBusinessCloudAppSecret
            | Self::WhatsappBusinessCloudWebhookVerifyToken => secret_kind == SecretKind::ApiToken,
        }
    }
}

impl TryFrom<&str> for ProviderAccountSecretPurpose {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "imap_password" => Ok(Self::ImapPassword),
            "smtp_password" => Ok(Self::SmtpPassword),
            "telegram_api_hash" => Ok(Self::TelegramApiHash),
            "telegram_session_key" => Ok(Self::TelegramSessionKey),
            "telegram_bot_token" => Ok(Self::TelegramBotToken),
            "whatsapp_web_session_key" => Ok(Self::WhatsappWebSessionKey),
            "whatsapp_business_cloud_access_token" => Ok(Self::WhatsappBusinessCloudAccessToken),
            "whatsapp_business_cloud_app_secret" => Ok(Self::WhatsappBusinessCloudAppSecret),
            "whatsapp_business_cloud_webhook_verify_token" => {
                Ok(Self::WhatsappBusinessCloudWebhookVerifyToken)
            }
            other => Err(CommunicationIngestionError::UnsupportedSecretPurpose(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
}

impl NewProviderAccountSecretBinding {
    pub fn new(
        account_id: impl Into<String>,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_ref: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            secret_purpose,
            secret_ref: secret_ref.into(),
        }
    }

    pub(in crate::domains::communications::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("secret_ref", &self.secret_ref)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}
