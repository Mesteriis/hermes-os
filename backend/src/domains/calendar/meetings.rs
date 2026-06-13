use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore,
    DecisionStoreError, NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationStore, ObligationStoreError,
};

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
        rows.into_iter().map(row_to_meeting_outcome).collect()
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
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO meeting_outcomes (event_id, outcome_type, title, description, owner_person_id, due_date) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at")
            .bind(event_id).bind(outcome_type).bind(title).bind(description).bind(owner_id).bind(due_date).fetch_one(&mut *transaction).await?;
        let mut outcome = row_to_meeting_outcome(row)?;

        if let Some(linked_entity_id) =
            Self::project_outcome_domain_record(&mut transaction, &outcome).await?
        {
            let row = sqlx::query(
                "UPDATE meeting_outcomes SET linked_entity_id = $1, updated_at = now() WHERE id::text = $2 RETURNING id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at",
            )
            .bind(linked_entity_id)
            .bind(&outcome.id)
            .fetch_one(&mut *transaction)
            .await?;
            outcome = row_to_meeting_outcome(row)?;
        }

        transaction.commit().await?;
        Ok(outcome)
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

    async fn project_outcome_domain_record(
        transaction: &mut Transaction<'_, Postgres>,
        outcome: &MeetingOutcome,
    ) -> Result<Option<String>, MeetingsError> {
        match outcome.outcome_type.as_str() {
            "decision" => {
                let description = outcome.description.as_deref().unwrap_or(&outcome.title);
                let decision = NewDecision::new(
                    outcome.title.clone(),
                    description,
                    outcome.confidence,
                    DecisionReviewState::Suggested,
                )
                .metadata(meeting_outcome_metadata(outcome));
                let evidence = NewDecisionEvidence::new(
                    DecisionEvidenceSourceKind::Event,
                    outcome.event_id.clone(),
                )
                .quote(description)
                .confidence(outcome.confidence)
                .metadata(meeting_outcome_metadata(outcome));
                let impact =
                    NewDecisionImpactedEntity::new(DecisionEntityKind::Event, &outcome.event_id)
                        .impact_type("meeting_outcome")
                        .metadata(meeting_outcome_metadata(outcome));
                let stored = DecisionStore::upsert_with_evidence_in_transaction(
                    transaction,
                    &decision,
                    &[evidence],
                    &[impact],
                )
                .await?;

                Ok(Some(stored.decision_id))
            }
            "task" | "promise" | "follow_up" => {
                let description = outcome.description.as_deref().unwrap_or(&outcome.title);
                let (obligated_entity_kind, obligated_entity_id) = outcome
                    .owner_person_id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(|owner_person_id| {
                        (ObligationEntityKind::Persona, owner_person_id.to_owned())
                    })
                    .unwrap_or_else(|| (ObligationEntityKind::Event, outcome.event_id.clone()));
                let mut obligation = NewObligation::new(
                    obligated_entity_kind,
                    obligated_entity_id,
                    outcome.title.clone(),
                    outcome.confidence,
                    ObligationReviewState::Suggested,
                )
                .metadata(meeting_outcome_metadata(outcome));
                if let Some(due_date) = outcome.due_date {
                    obligation = obligation.due_at(due_date);
                }
                let evidence = NewObligationEvidence::new(
                    ObligationEvidenceSourceKind::Event,
                    outcome.event_id.clone(),
                )
                .quote(description)
                .confidence(outcome.confidence)
                .metadata(meeting_outcome_metadata(outcome));
                let stored = ObligationStore::upsert_with_evidence_in_transaction(
                    transaction,
                    &obligation,
                    &[evidence],
                )
                .await?;

                Ok(Some(stored.obligation_id))
            }
            _ => Ok(None),
        }
    }
}

fn row_to_meeting_outcome(row: PgRow) -> Result<MeetingOutcome, MeetingsError> {
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

fn meeting_outcome_metadata(outcome: &MeetingOutcome) -> Value {
    json!({
        "source": "meeting_outcome_adapter",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
        "outcome_type": outcome.outcome_type,
    })
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
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
    #[error("not found")]
    NotFound,
}
