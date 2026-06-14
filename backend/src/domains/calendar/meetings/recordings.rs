use sqlx::postgres::PgPool;

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
        let query = format!(
            "INSERT INTO event_recordings (event_id, file_path, source, duration_seconds) VALUES ($1,$2,$3,$4) RETURNING {EVENT_RECORDING_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(file_path)
            .bind(source.unwrap_or("manual"))
            .bind(duration_seconds)
            .fetch_one(&self.pool)
            .await?;
        row_to_event_recording(row)
    }
}
