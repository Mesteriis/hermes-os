use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CommunicationAccountError {
    #[error("unsupported communication provider kind: {0}")]
    UnsupportedProviderKind(String),

    #[error("unsupported provider account secret purpose: {0}")]
    UnsupportedSecretPurpose(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

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
    ZulipBot,
    ZoomUser,
    ZoomServerToServer,
    YandexTelemostUser,
}

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
            Self::ZulipBot => "zulip_bot",
            Self::ZoomUser => "zoom_user",
            Self::ZoomServerToServer => "zoom_server_to_server",
            Self::YandexTelemostUser => "yandex_telemost_user",
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

    pub fn is_zulip(self) -> bool {
        matches!(self, Self::ZulipBot)
    }

    pub fn is_zoom(self) -> bool {
        matches!(self, Self::ZoomUser | Self::ZoomServerToServer)
    }

    pub fn is_yandex_telemost(self) -> bool {
        matches!(self, Self::YandexTelemostUser)
    }
}

impl TryFrom<&str> for CommunicationProviderKind {
    type Error = CommunicationAccountError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            "whatsapp_business_cloud" => Ok(Self::WhatsappBusinessCloud),
            "zulip_bot" => Ok(Self::ZulipBot),
            "zoom_user" => Ok(Self::ZoomUser),
            "zoom_server_to_server" => Ok(Self::ZoomServerToServer),
            "yandex_telemost_user" => Ok(Self::YandexTelemostUser),
            other => Err(CommunicationAccountError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderAccount {
    pub fn is_deleted(&self) -> bool {
        self.config
            .get("auth_state")
            .and_then(Value::as_str)
            .is_some_and(|state| state == "deleted")
            || self.config.get("deleted_at").is_some()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountUsage {
    pub raw_record_count: i64,
    pub message_count: i64,
    pub checkpoint_count: i64,
}

impl ProviderAccountUsage {
    pub fn has_retained_evidence(&self) -> bool {
        self.raw_record_count > 0 || self.message_count > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeletedProviderAccount {
    pub account: Option<ProviderAccount>,
    pub unbound_secret_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
}

impl NewProviderAccount {
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: CommunicationProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            config: json!({}),
        }
    }

    pub fn config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }

    pub fn validate(&self) -> Result<(), CommunicationAccountError> {
        required_non_empty("account_id", &self.account_id)?;
        required_non_empty("display_name", &self.display_name)?;
        required_non_empty("external_account_id", &self.external_account_id)?;
        required_object("config", &self.config)
    }
}

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
    ZulipApiKey,
    ZoomOauthToken,
    ZoomClientSecret,
    ZoomWebhookSecret,
    YandexTelemostOauthToken,
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
            Self::ZulipApiKey => "zulip_api_key",
            Self::ZoomOauthToken => "zoom_oauth_token",
            Self::ZoomClientSecret => "zoom_client_secret",
            Self::ZoomWebhookSecret => "zoom_webhook_secret",
            Self::YandexTelemostOauthToken => "yandex_telemost_oauth_token",
        }
    }

    pub fn accepts_secret_kind(self, secret_kind: impl SecretKindTag) -> bool {
        match self {
            Self::OauthToken | Self::ZoomOauthToken | Self::YandexTelemostOauthToken => {
                secret_kind.secret_kind_tag() == "oauth_token"
            }
            Self::ImapPassword | Self::SmtpPassword => {
                matches!(secret_kind.secret_kind_tag(), "app_password" | "password")
            }
            Self::TelegramApiHash
            | Self::TelegramBotToken
            | Self::WhatsappBusinessCloudAccessToken
            | Self::WhatsappBusinessCloudAppSecret
            | Self::WhatsappBusinessCloudWebhookVerifyToken
            | Self::ZulipApiKey
            | Self::ZoomClientSecret
            | Self::ZoomWebhookSecret => secret_kind.secret_kind_tag() == "api_token",
            Self::TelegramSessionKey | Self::WhatsappWebSessionKey => {
                matches!(secret_kind.secret_kind_tag(), "private_key" | "other")
            }
        }
    }
}

pub trait SecretKindTag {
    fn secret_kind_tag(self) -> &'static str;
}

impl TryFrom<&str> for ProviderAccountSecretPurpose {
    type Error = CommunicationAccountError;

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
            "zulip_api_key" => Ok(Self::ZulipApiKey),
            "zoom_oauth_token" => Ok(Self::ZoomOauthToken),
            "zoom_client_secret" => Ok(Self::ZoomClientSecret),
            "zoom_webhook_secret" => Ok(Self::ZoomWebhookSecret),
            "yandex_telemost_oauth_token" => Ok(Self::YandexTelemostOauthToken),
            other => Err(CommunicationAccountError::UnsupportedSecretPurpose(
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

    pub fn validate(&self) -> Result<(), CommunicationAccountError> {
        required_non_empty("account_id", &self.account_id)?;
        required_non_empty("secret_ref", &self.secret_ref)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderAccountMutationOrigin {
    LocalRuntime,
    VaultSource,
}

#[derive(Debug, Error)]
#[error("communication provider account port error: {0}")]
pub struct ProviderAccountPortError(pub String);

impl ProviderAccountPortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

#[derive(Debug, Error)]
#[error("communication provider secret binding port error: {0}")]
pub struct ProviderSecretBindingPortError(pub String);

impl ProviderSecretBindingPortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub trait ProviderAccountLookupPort: Send + Sync {
    fn get<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn list<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<ProviderAccount>, ProviderAccountPortError>> + Send + 'a,
        >,
    >;
}

pub trait ProviderAccountCommandPort: ProviderAccountLookupPort {
    fn upsert<'a>(
        &'a self,
        account: &'a NewProviderAccount,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccount, ProviderAccountPortError>> + Send + 'a>>;

    fn upsert_runtime_account<'a>(
        &'a self,
        account_id: String,
        provider_kind: String,
        display_name: String,
        external_account_id: String,
        config: Value,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccount, ProviderAccountPortError>> + Send + 'a>>;

    fn update_config<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a Value,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn update_config_with_origin<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a Value,
        origin: ProviderAccountMutationOrigin,
        actor: &'a str,
        action: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn mark_logged_out<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;

    fn delete_metadata<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<DeletedProviderAccount, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    >;
}

pub trait ProviderSecretBindingLookupPort: Send + Sync {
    fn list_for_account<'a>(
        &'a self,
        account_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn get_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;
}

pub trait ProviderSecretBindingCommandPort: ProviderSecretBindingLookupPort {
    fn bind<'a>(
        &'a self,
        binding: &'a NewProviderAccountSecretBinding,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<ProviderAccountSecretBinding, ProviderSecretBindingPortError>,
                > + Send
                + 'a,
        >,
    >;

    fn unbind_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    >;
}

fn required_non_empty(field: &'static str, value: &str) -> Result<(), CommunicationAccountError> {
    if value.trim().is_empty() {
        return Err(CommunicationAccountError::EmptyField(field));
    }
    Ok(())
}

fn required_object(field: &'static str, value: &Value) -> Result<(), CommunicationAccountError> {
    if !value.is_object() {
        return Err(CommunicationAccountError::NonObjectJson(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CommunicationAccountError, CommunicationProviderKind, NewProviderAccount};

    #[test]
    fn provider_kind_preserves_persisted_strings() {
        assert_eq!(
            CommunicationProviderKind::try_from("zulip_bot"),
            Ok(CommunicationProviderKind::ZulipBot)
        );
        assert_eq!(CommunicationProviderKind::ZulipBot.as_str(), "zulip_bot");
    }

    #[test]
    fn provider_account_requires_identity_and_object_config() {
        let account = NewProviderAccount::new(
            "account-1",
            CommunicationProviderKind::Gmail,
            "Gmail",
            "external-1",
        )
        .config(json!(false));
        assert_eq!(
            account.validate(),
            Err(CommunicationAccountError::NonObjectJson("config"))
        );
    }
}
