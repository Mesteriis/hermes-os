use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

// ── MeetingNote ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingNote {
    pub id: String,
    pub event_id: String,
    pub content: String,
    pub format: String,
    pub source: String,
    pub linked_note_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct MeetingNoteStore {
    pool: PgPool,
}

impl MeetingNoteStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingNote>, MeetingsError> {
        let rows = sqlx::query("SELECT id::text, event_id, content, format, source, linked_note_id, created_at, updated_at FROM meeting_notes WHERE event_id=$1 ORDER BY created_at DESC")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(MeetingNote {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    content: r.try_get("content")?,
                    format: r.try_get("format")?,
                    source: r.try_get("source")?,
                    linked_note_id: r.try_get("linked_note_id")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        event_id: &str,
        content: &str,
        format: Option<&str>,
        source: Option<&str>,
    ) -> Result<MeetingNote, MeetingsError> {
        let row = sqlx::query("INSERT INTO meeting_notes (event_id, content, format, source) VALUES ($1,$2,$3,$4) RETURNING id::text, event_id, content, format, source, linked_note_id, created_at, updated_at")
            .bind(event_id).bind(content).bind(format.unwrap_or("markdown")).bind(source.unwrap_or("manual")).fetch_one(&self.pool).await?;
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
}

// ── MeetingOutcome ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingOutcome {
    pub id: String,
    pub event_id: String,
    pub outcome_type: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_person_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub source: String,
    pub confidence: f64,
    pub linked_entity_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct MeetingOutcomeStore {
    pool: PgPool,
}

impl MeetingOutcomeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingOutcome>, MeetingsError> {
        let rows = sqlx::query("SELECT id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at FROM meeting_outcomes WHERE event_id=$1 ORDER BY outcome_type, title")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(MeetingOutcome {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    outcome_type: r.try_get("outcome_type")?,
                    title: r.try_get("title")?,
                    description: r.try_get("description")?,
                    owner_person_id: r.try_get("owner_person_id")?,
                    due_date: r.try_get("due_date")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    linked_entity_id: r.try_get("linked_entity_id")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let row = sqlx::query("INSERT INTO meeting_outcomes (event_id, outcome_type, title, description, owner_person_id, due_date) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at")
            .bind(event_id).bind(outcome_type).bind(title).bind(description).bind(owner_id).bind(due_date).fetch_one(&self.pool).await?;
        Ok(MeetingOutcome {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            outcome_type: row.try_get("outcome_type")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            owner_person_id: row.try_get("owner_person_id")?,
            due_date: row.try_get("due_date")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            linked_entity_id: row.try_get("linked_entity_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn follow_up_status(&self, event_id: &str) -> Result<Value, MeetingsError> {
        let rows = sqlx::query("SELECT outcome_type, COUNT(*) as cnt FROM meeting_outcomes WHERE event_id=$1 GROUP BY outcome_type")
            .bind(event_id).fetch_all(&self.pool).await?;
        let mut status = serde_json::Map::new();
        for r in &rows {
            let t: String = r.try_get("outcome_type")?;
            let c: i64 = r.try_get("cnt")?;
            status.insert(t, serde_json::Value::Number(c.into()));
        }
        Ok(Value::Object(status))
    }
}

// ── EventRecording ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecording {
    pub id: String,
    pub event_id: String,
    pub file_path: Option<String>,
    pub source: String,
    pub duration_seconds: Option<i32>,
    pub transcript_id: Option<String>,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventRecordingStore {
    pool: PgPool,
}

impl EventRecordingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventRecording>, MeetingsError> {
        let rows = sqlx::query("SELECT id::text, event_id, file_path, source, duration_seconds, transcript_id::text, processing_status, created_at, updated_at FROM event_recordings WHERE event_id=$1")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(EventRecording {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    file_path: r.try_get("file_path")?,
                    source: r.try_get("source")?,
                    duration_seconds: r.try_get("duration_seconds")?,
                    transcript_id: r.try_get("transcript_id")?,
                    processing_status: r.try_get("processing_status")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
    ) -> Result<EventRecording, MeetingsError> {
        let row = sqlx::query("INSERT INTO event_recordings (event_id, file_path, source, duration_seconds) VALUES ($1,$2,$3,$4) RETURNING id::text, event_id, file_path, source, duration_seconds, transcript_id::text, processing_status, created_at, updated_at")
            .bind(event_id).bind(file_path).bind(source.unwrap_or("manual")).bind(duration_seconds).fetch_one(&self.pool).await?;
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
}

// ── EventTranscript ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventTranscript {
    pub id: String,
    pub event_id: String,
    pub text: String,
    pub language: String,
    pub summary: Option<String>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventTranscriptStore {
    pool: PgPool,
}

impl EventTranscriptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventTranscript>, MeetingsError> {
        let row = sqlx::query("SELECT id::text, event_id, text, language, summary, model, created_at FROM event_transcripts WHERE event_id=$1 ORDER BY created_at DESC LIMIT 1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(EventTranscript {
                id: r.try_get("id")?,
                event_id: r.try_get("event_id")?,
                text: r.try_get("text")?,
                language: r.try_get("language")?,
                summary: r.try_get("summary")?,
                model: r.try_get("model")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .transpose()
    }
}

#[derive(Debug, Error)]
pub enum MeetingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
