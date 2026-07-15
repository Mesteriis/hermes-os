use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
