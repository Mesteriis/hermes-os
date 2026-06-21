use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::CallError;
use super::validation::{validate_array, validate_non_empty, validate_object};

#[derive(Clone, Debug, PartialEq)]
pub struct NewTelegramCall {
    pub call_id: String,
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: CallDirection,
    pub call_state: CallState,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transcription_policy_id: Option<String>,
    pub metadata: Value,
}

impl NewTelegramCall {
    pub(super) fn validate(&self) -> Result<(), CallError> {
        validate_non_empty("call_id", &self.call_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_call_id", &self.provider_call_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TelegramCall {
    pub call_id: String,
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: String,
    pub call_state: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transcription_policy_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallDirection {
    Incoming,
    Outgoing,
}

impl CallDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Outgoing => "outgoing",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    Ringing,
    Active,
    Ended,
    Missed,
    Declined,
    Failed,
}

impl CallState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ringing => "ringing",
            Self::Active => "active",
            Self::Ended => "ended",
            Self::Missed => "missed",
            Self::Declined => "declined",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCallTranscript {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub transcript_status: TranscriptStatus,
    pub stt_provider: String,
    pub source_audio_ref: Option<String>,
    pub language_code: Option<String>,
    pub transcript_text: String,
    pub segments: Value,
    pub provenance: Value,
}

impl NewCallTranscript {
    pub(super) fn validate(&self) -> Result<(), CallError> {
        validate_non_empty("transcript_id", &self.transcript_id)?;
        validate_non_empty("call_id", &self.call_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("stt_provider", &self.stt_provider)?;
        validate_array("segments", &self.segments)?;
        validate_object("provenance", &self.provenance)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CallTranscript {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub transcript_status: String,
    pub stt_provider: String,
    pub source_audio_ref: Option<String>,
    pub language_code: Option<String>,
    pub transcript_text: String,
    pub segments: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl TranscriptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}
