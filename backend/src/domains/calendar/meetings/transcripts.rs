use sqlx::postgres::PgPool;

use super::rows::{EVENT_TRANSCRIPT_COLUMNS, row_to_event_transcript};
use super::{EventTranscript, MeetingsError};

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
}
