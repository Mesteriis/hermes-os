use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::{
    link_calendar_entity, link_calendar_entity_in_transaction,
};

use super::rows::{EVENT_RECORDING_COLUMNS, row_to_event_recording};
use super::{EventRecording, MeetingsError};

#[derive(Clone)]
pub struct EventRecordingStore {
    pool: PgPool,
}

impl EventRecordingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventRecording>, MeetingsError> {
        let query =
            format!("SELECT {EVENT_RECORDING_COLUMNS} FROM event_recordings WHERE event_id=$1");
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_event_recording).collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
    ) -> Result<EventRecording, MeetingsError> {
        self.add_with_observation(event_id, file_path, source, duration_seconds, None)
            .await
    }

    pub async fn add_with_observation(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            file_path,
            source,
            duration_seconds,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(recording)
    }

    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let query = format!(
            "INSERT INTO event_recordings (event_id, file_path, source, duration_seconds) VALUES ($1,$2,$3,$4) RETURNING {EVENT_RECORDING_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(file_path)
            .bind(source.unwrap_or("manual"))
            .bind(duration_seconds)
            .fetch_one(&mut **transaction)
            .await?;
        let recording = row_to_event_recording(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_recording",
                recording.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "duration_seconds": recording.duration_seconds,
                }),
                None,
            )
            .await?;
        }
        Ok(recording)
    }

    pub(crate) async fn find_by_file_path_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: &str,
    ) -> Result<Option<EventRecording>, MeetingsError> {
        let query = format!(
            "SELECT {EVENT_RECORDING_COLUMNS} FROM event_recordings WHERE event_id=$1 AND file_path=$2 ORDER BY created_at DESC LIMIT 1"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(file_path)
            .fetch_optional(&mut **transaction)
            .await?;
        row.map(row_to_event_recording).transpose()
    }

    pub(crate) async fn attach_transcript_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        recording_id: &str,
        transcript_id: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let query = format!(
            "UPDATE event_recordings SET transcript_id=$2, processing_status='transcribed', updated_at=now() WHERE id::text=$1 RETURNING {EVENT_RECORDING_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(recording_id)
            .bind(transcript_id)
            .fetch_one(&mut **transaction)
            .await?;
        let recording = row_to_event_recording(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_recording",
                recording.id.clone(),
                Some("transcript_attached"),
                serde_json::json!({
                    "event_id": recording.event_id,
                    "transcript_id": transcript_id,
                    "processing_status": recording.processing_status,
                }),
                None,
            )
            .await?;
        }
        Ok(recording)
    }

    pub async fn find_by_file_path(
        &self,
        event_id: &str,
        file_path: &str,
    ) -> Result<Option<EventRecording>, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording =
            Self::find_by_file_path_in_transaction(&mut transaction, event_id, file_path).await?;
        transaction.rollback().await?;
        Ok(recording)
    }

    pub async fn attach_transcript(
        &self,
        recording_id: &str,
        transcript_id: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording = Self::attach_transcript_in_transaction(
            &mut transaction,
            recording_id,
            transcript_id,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(recording)
    }
}
