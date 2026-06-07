use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone)]
pub struct CallIntelligenceStore {
    pool: PgPool,
}

impl CallIntelligenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_call(&self, call: &NewTelegramCall) -> Result<TelegramCall, CallError> {
        call.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_calls (
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (account_id, provider_call_id)
            DO UPDATE SET
                provider_chat_id = EXCLUDED.provider_chat_id,
                direction = EXCLUDED.direction,
                call_state = EXCLUDED.call_state,
                started_at = EXCLUDED.started_at,
                ended_at = EXCLUDED.ended_at,
                transcription_policy_id = EXCLUDED.transcription_policy_id,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(call.call_id.trim())
        .bind(call.account_id.trim())
        .bind(call.provider_call_id.trim())
        .bind(call.provider_chat_id.trim())
        .bind(call.direction.as_str())
        .bind(call.call_state.as_str())
        .bind(call.started_at)
        .bind(call.ended_at)
        .bind(call.transcription_policy_id.as_deref())
        .bind(&call.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_call(row)
    }

    pub async fn upsert_transcript(
        &self,
        transcript: &NewCallTranscript,
    ) -> Result<CallTranscript, CallError> {
        transcript.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO call_transcripts (
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (transcript_id)
            DO UPDATE SET
                transcript_status = EXCLUDED.transcript_status,
                stt_provider = EXCLUDED.stt_provider,
                source_audio_ref = EXCLUDED.source_audio_ref,
                language_code = EXCLUDED.language_code,
                transcript_text = EXCLUDED.transcript_text,
                segments = EXCLUDED.segments,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            RETURNING
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                created_at,
                updated_at
            "#,
        )
        .bind(transcript.transcript_id.trim())
        .bind(transcript.call_id.trim())
        .bind(transcript.account_id.trim())
        .bind(transcript.provider_chat_id.trim())
        .bind(transcript.transcript_status.as_str())
        .bind(transcript.stt_provider.trim())
        .bind(transcript.source_audio_ref.as_deref())
        .bind(transcript.language_code.as_deref())
        .bind(&transcript.transcript_text)
        .bind(&transcript.segments)
        .bind(&transcript.provenance)
        .fetch_one(&self.pool)
        .await?;

        row_to_transcript(row)
    }

    pub async fn list_calls(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramCall>, CallError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                created_at,
                updated_at
            FROM telegram_calls
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY COALESCE(started_at, created_at) DESC, call_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_call).collect()
    }

    pub async fn transcript_for_call(
        &self,
        call_id: &str,
    ) -> Result<Option<CallTranscript>, CallError> {
        validate_non_empty("call_id", call_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                created_at,
                updated_at
            FROM call_transcripts
            WHERE call_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(call_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_transcript).transpose()
    }
}

pub trait SpeechToTextProvider {
    fn provider_name(&self) -> &'static str;
    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError>;
}

pub struct FixtureSpeechToTextProvider;

impl SpeechToTextProvider for FixtureSpeechToTextProvider {
    fn provider_name(&self) -> &'static str {
        "fixture-stt"
    }

    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError> {
        validate_non_empty("audio_ref", audio_ref)?;
        Ok(FixtureTranscript {
            text: format!("Fixture transcript for {audio_ref}: follow up on the Telegram call."),
            segments: json!([
                {
                    "speaker": "local",
                    "start_ms": 0,
                    "end_ms": 2400,
                    "text": "follow up on the Telegram call"
                }
            ]),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureTranscript {
    pub text: String,
    pub segments: Value,
}

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
    fn validate(&self) -> Result<(), CallError> {
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
    fn validate(&self) -> Result<(), CallError> {
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

#[derive(Debug, Error)]
pub enum CallError {
    #[error("invalid call intelligence request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn row_to_call(row: PgRow) -> Result<TelegramCall, CallError> {
    Ok(TelegramCall {
        call_id: row.try_get("call_id")?,
        account_id: row.try_get("account_id")?,
        provider_call_id: row.try_get("provider_call_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        direction: row.try_get("direction")?,
        call_state: row.try_get("call_state")?,
        started_at: row.try_get("started_at")?,
        ended_at: row.try_get("ended_at")?,
        transcription_policy_id: row.try_get("transcription_policy_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_transcript(row: PgRow) -> Result<CallTranscript, CallError> {
    Ok(CallTranscript {
        transcript_id: row.try_get("transcript_id")?,
        call_id: row.try_get("call_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        transcript_status: row.try_get("transcript_status")?,
        stt_provider: row.try_get("stt_provider")?,
        source_audio_ref: row.try_get("source_audio_ref")?,
        language_code: row.try_get("language_code")?,
        transcript_text: row.try_get("transcript_text")?,
        segments: row.try_get("segments")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn validate_limit(limit: i64) -> Result<i64, CallError> {
    if !(1..=100).contains(&limit) {
        return Err(CallError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, CallError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_object() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

fn validate_array(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_array() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
