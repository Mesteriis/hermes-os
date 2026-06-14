use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;
use super::chats::TelegramChatKind;

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

    pub(in crate::integrations::telegram::client) fn validate_for_runtime(
        &self,
        runtime_kind: &str,
    ) -> Result<(), TelegramError> {
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

    pub(in crate::integrations::telegram::client) fn source_fingerprint(&self) -> String {
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
