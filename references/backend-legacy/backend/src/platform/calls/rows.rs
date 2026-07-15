use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::CallError;
use super::models::{CallTranscript, TelegramCall};

pub(super) fn row_to_call(row: PgRow) -> Result<TelegramCall, CallError> {
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

pub(super) fn row_to_transcript(row: PgRow) -> Result<CallTranscript, CallError> {
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
