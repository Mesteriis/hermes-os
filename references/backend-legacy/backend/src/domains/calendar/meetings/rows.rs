use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::MeetingsError;
use super::models::{EventRecording, EventTranscript, MeetingNote, MeetingOutcome};

pub(super) const MEETING_NOTE_COLUMNS: &str =
    "id::text, event_id, content, format, source, linked_note_id, created_at, updated_at";
pub(super) const MEETING_OUTCOME_COLUMNS: &str = "id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at";
pub(super) const EVENT_RECORDING_COLUMNS: &str = "id::text, event_id, file_path, source, duration_seconds, transcript_id::text, processing_status, created_at, updated_at";
pub(super) const EVENT_TRANSCRIPT_COLUMNS: &str =
    "id::text, event_id, text, language, summary, model, created_at";

pub(super) fn row_to_meeting_note(row: PgRow) -> Result<MeetingNote, MeetingsError> {
    Ok(MeetingNote {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        content: row.try_get("content")?,
        format: row.try_get("format")?,
        source: row.try_get("source")?,
        linked_note_id: row.try_get("linked_note_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_meeting_outcome(row: PgRow) -> Result<MeetingOutcome, MeetingsError> {
    Ok(MeetingOutcome {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        outcome_type: row.try_get("outcome_type")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        owner_person_id: row.try_get("owner_person_id")?,
        due_date: row.try_get("due_date")?,
        source: row.try_get("source")?,
        confidence: f64::from(row.try_get::<f32, _>("confidence")?),
        linked_entity_id: row.try_get("linked_entity_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_event_recording(row: PgRow) -> Result<EventRecording, MeetingsError> {
    Ok(EventRecording {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        file_path: row.try_get("file_path")?,
        source: row.try_get("source")?,
        duration_seconds: row.try_get("duration_seconds")?,
        transcript_id: row.try_get("transcript_id")?,
        processing_status: row.try_get("processing_status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_event_transcript(row: PgRow) -> Result<EventTranscript, MeetingsError> {
    Ok(EventTranscript {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        text: row.try_get("text")?,
        language: row.try_get("language")?,
        summary: row.try_get("summary")?,
        model: row.try_get("model")?,
        created_at: row.try_get("created_at")?,
    })
}
