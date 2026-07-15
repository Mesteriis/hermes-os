use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;

use super::errors::MeetingsError;
use super::models::EventTranscript;
use super::rows::{EVENT_TRANSCRIPT_COLUMNS, row_to_event_transcript};

#[derive(Clone)]
pub struct EventTranscriptStore {
    pool: PgPool,
}

impl EventTranscriptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventTranscript>, MeetingsError> {
        let query = format!(
            "SELECT {EVENT_TRANSCRIPT_COLUMNS} FROM event_transcripts WHERE event_id=$1 ORDER BY created_at DESC LIMIT 1"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_event_transcript).transpose()
    }

    pub async fn add_with_observation(
        &self,
        event_id: &str,
        text: &str,
        language: Option<&str>,
        summary: Option<&str>,
        model: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<EventTranscript, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let transcript = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            text,
            language,
            summary,
            model,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(transcript)
    }

    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        text: &str,
        language: Option<&str>,
        summary: Option<&str>,
        model: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<EventTranscript, MeetingsError> {
        let query = format!(
            "INSERT INTO event_transcripts (event_id, text, language, summary, model) VALUES ($1,$2,$3,$4,$5) RETURNING {EVENT_TRANSCRIPT_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(text)
            .bind(language.unwrap_or("en"))
            .bind(summary)
            .bind(model)
            .fetch_one(&mut **transaction)
            .await?;
        let transcript = row_to_event_transcript(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_transcript",
                transcript.id.clone(),
                Some("transcript_projection"),
                serde_json::json!({
                    "event_id": event_id,
                    "language": transcript.language,
                    "model": transcript.model,
                }),
                None,
            )
            .await?;
        }
        Ok(transcript)
    }
}
