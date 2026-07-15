use sqlx::postgres::PgPool;

use super::errors::CallError;
use super::models::{CallTranscript, NewCallTranscript, NewTelegramCall, TelegramCall};
use super::rows::{row_to_call, row_to_transcript};
use super::validation::{validate_limit, validate_non_empty};

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
        provider_chat_id: Option<&str>,
        provider: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramCall>, CallError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let provider = provider.map(str::trim).filter(|value| !value.is_empty());
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
              AND ($2::text IS NULL OR provider_chat_id = $2)
              AND ($3::text IS NULL OR metadata ->> 'provider' = $3)
            ORDER BY COALESCE(started_at, created_at) DESC, call_id ASC
            LIMIT $4
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(provider)
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

    pub async fn list_expired_transcripts(
        &self,
        account_id: &str,
        provider: &str,
        limit: i64,
    ) -> Result<Vec<CallTranscript>, CallError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("provider", provider)?;
        let limit = validate_limit(limit.clamp(1, 500))?;
        let rows = sqlx::query(
            r#"
            SELECT
                transcript.transcript_id,
                transcript.call_id,
                transcript.account_id,
                transcript.provider_chat_id,
                transcript.transcript_status,
                transcript.stt_provider,
                transcript.source_audio_ref,
                transcript.language_code,
                transcript.transcript_text,
                transcript.segments,
                transcript.provenance,
                transcript.created_at,
                transcript.updated_at
            FROM call_transcripts transcript
            JOIN telegram_calls call_evidence ON call_evidence.call_id = transcript.call_id
            WHERE transcript.account_id = $1
              AND call_evidence.metadata ->> 'provider' = $2
              AND NULLIF(transcript.provenance -> 'retention_policy' ->> 'expires_at', '') IS NOT NULL
              AND (transcript.provenance -> 'retention_policy' ->> 'expires_at')::timestamptz <= now()
            ORDER BY (transcript.provenance -> 'retention_policy' ->> 'expires_at')::timestamptz ASC,
                     transcript.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(account_id.trim())
        .bind(provider.trim())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_transcript).collect()
    }

    pub async fn remove_transcript(
        &self,
        transcript_id: &str,
    ) -> Result<Option<CallTranscript>, CallError> {
        validate_non_empty("transcript_id", transcript_id)?;
        let row = sqlx::query(
            r#"
            DELETE FROM call_transcripts
            WHERE transcript_id = $1
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
        .bind(transcript_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_transcript).transpose()
    }
}
