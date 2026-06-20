use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::platform::communications::CommunicationProviderKind;

use super::errors::WhatsappWebError;
use super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsappWebAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: String,
    pub local_state_path: String,
}

impl WhatsappWebAccountSetupRequest {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub session: WhatsappWebSession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewWhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: WhatsappWebCompanionRuntime,
    pub link_state: WhatsappWebLinkState,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewWhatsappWebSession {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("session_id", &self.session_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: String,
    pub link_state: String,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebCompanionRuntime {
    Fixture,
    ManualWebview,
    Blocked,
}

impl WhatsappWebCompanionRuntime {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::ManualWebview => "manual_webview",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebLinkState {
    Fixture,
    QrPending,
    Linked,
    Degraded,
    Revoked,
    Blocked,
}

impl WhatsappWebLinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::QrPending => "qr_pending",
            Self::Linked => "linked",
            Self::Degraded => "degraded",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: WhatsappWebDeliveryState,
}

impl NewWhatsappWebMessage {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
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
pub enum WhatsappWebDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl WhatsappWebDeliveryState {
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

impl TryFrom<String> for WhatsappWebDeliveryState {
    type Error = WhatsappWebError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported WhatsApp Web delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessage {
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
