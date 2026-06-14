use sqlx::postgres::PgPool;

use super::rows::{MEETING_NOTE_COLUMNS, row_to_meeting_note};
use super::{MeetingNote, MeetingsError};

#[derive(Clone)]
pub struct MeetingNoteStore {
    pool: PgPool,
}

impl MeetingNoteStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingNote>, MeetingsError> {
        let query = format!(
            "SELECT {MEETING_NOTE_COLUMNS} FROM meeting_notes WHERE event_id=$1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_meeting_note).collect()
    }

    pub async fn create(
        &self,
        event_id: &str,
        content: &str,
        format: Option<&str>,
        source: Option<&str>,
    ) -> Result<MeetingNote, MeetingsError> {
        let query = format!(
            "INSERT INTO meeting_notes (event_id, content, format, source) VALUES ($1,$2,$3,$4) RETURNING {MEETING_NOTE_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(content)
            .bind(format.unwrap_or("markdown"))
            .bind(source.unwrap_or("manual"))
            .fetch_one(&self.pool)
            .await?;
        row_to_meeting_note(row)
    }
}
