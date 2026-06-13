use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::domains::mail::core::CommunicationProviderKind;
use crate::platform::secrets::{SecretKind, SecretStoreKind};

use super::errors::TelegramError;
use super::validation::{required_optional_value, validate_non_empty, validate_object};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramAccountSetupRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub bot_token: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub qr_authorized: bool,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramLiveAccountSetupRequest {
    pub(crate) fn with_inferred_qr_authorization(mut self) -> Self {
        if self.is_finalized_qr_user_account() {
            self.qr_authorized = true;
        }
        self
    }

    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.is_qr_authorized_user_account() {
            return self;
        }
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(super) fn is_qr_authorized_user_account(&self) -> bool {
        self.qr_authorized && self.provider_kind == CommunicationProviderKind::TelegramUser
    }

    pub(super) fn is_finalized_qr_user_account(&self) -> bool {
        self.provider_kind == CommunicationProviderKind::TelegramUser
            && self
                .external_account_id
                .trim()
                .strip_prefix("telegram:")
                .is_some_and(|provider_user_id| !provider_user_id.trim().is_empty())
            && self
                .tdlib_data_path
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        match self.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if self.is_qr_authorized_user_account() {
                    required_optional_value("tdlib_data_path", self.tdlib_data_path.as_deref())?;
                    return Ok(());
                }
                let api_id = self.api_id.ok_or_else(|| {
                    TelegramError::InvalidRequest("api_id must not be empty".to_owned())
                })?;
                if api_id <= 0 {
                    return Err(TelegramError::InvalidRequest(
                        "api_id must be greater than zero".to_owned(),
                    ));
                }
                required_optional_value("api_hash", self.api_hash.as_deref())?;
            }
            CommunicationProviderKind::TelegramBot => {
                if self.qr_authorized {
                    return Err(TelegramError::InvalidRequest(
                        "qr_authorized is only supported for telegram_user".to_owned(),
                    ));
                }
                required_optional_value("bot_token", self.bot_token.as_deref())?;
            }
            _ => {
                return Err(TelegramError::InvalidRequest(
                    "provider_kind must be telegram_user or telegram_bot".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginStartRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramQrLoginStartRequest {
    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(crate) fn required_api_id(&self) -> Result<i64, TelegramError> {
        let api_id = self
            .api_id
            .ok_or_else(|| TelegramError::InvalidRequest("api_id must not be empty".to_owned()))?;
        if api_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "api_id must be greater than zero".to_owned(),
            ));
        }
        Ok(api_id)
    }

    pub(crate) fn required_api_hash(&self) -> Result<&str, TelegramError> {
        self.api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| TelegramError::InvalidRequest("api_hash must not be empty".to_owned()))
    }

    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        self.required_api_id()?;
        self.required_api_hash()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginPasswordRequest {
    pub password: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramQrLoginStatus {
    WaitingQrScan,
    WaitingPassword,
    Ready,
    Expired,
    Failed,
    RuntimeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramQrLoginStatusResponse {
    pub setup_id: String,
    pub account_id: String,
    pub status: TelegramQrLoginStatus,
    pub qr_link: Option<String>,
    pub qr_svg: Option<String>,
    pub telegram_user_id: Option<String>,
    pub telegram_username: Option<String>,
    pub suggested_account_id: Option<String>,
    pub suggested_display_name: Option<String>,
    pub suggested_external_account_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub poll_after_ms: u64,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub transcription_enabled: bool,
    pub credential_bindings: Vec<TelegramCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub runtime: String,
    pub lifecycle_state: String,
    pub transcription_enabled: bool,
    pub tdlib_data_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountListResponse {
    pub items: Vec<TelegramAccount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountLifecycleResponse {
    pub account: TelegramAccount,
    pub stopped_runtime_actor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramCredentialBinding {
    pub secret_purpose: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChat {
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: String,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChat {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: TelegramSyncState,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewTelegramChat {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

impl TelegramChatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Bot => "bot",
        }
    }
}

impl TryFrom<&str> for TelegramChatKind {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "private" => Ok(Self::Private),
            "group" => Ok(Self::Group),
            "channel" => Ok(Self::Channel),
            "bot" => Ok(Self::Bot),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram chat_kind `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramSyncState {
    Fixture,
    Syncing,
    Synced,
    Degraded,
    Error,
}

impl TelegramSyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Syncing => "syncing",
            Self::Synced => "synced",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewTelegramMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_kind: TelegramChatKind,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: TelegramDeliveryState,
}

impl NewTelegramMessage {
    fn validate(&self) -> Result<(), TelegramError> {
        self.validate_common()?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }

    pub(super) fn validate_for_runtime(&self, runtime_kind: &str) -> Result<(), TelegramError> {
        if runtime_kind == "tdlib" {
            self.validate_common()
        } else {
            self.validate()
        }
    }

    fn validate_common(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(super) fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl TelegramDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Sent => "sent",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for TelegramDeliveryState {
    type Error = TelegramError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramManualSendRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub text: String,
}

impl TelegramManualSendRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramManualSendResponse {
    pub raw_record_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub delivery_state: String,
    pub status: String,
    pub runtime_kind: String,
    pub rendered_preview_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramAttachmentAnchor {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
}
