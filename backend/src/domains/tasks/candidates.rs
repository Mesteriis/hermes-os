// This file exceeds 700 lines because it groups the task candidate store
// with candidate evidence, review state management, and related query types.
// These share tight coupling through the task candidate lifecycle state
// machine and evidence provenance tracking.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::domains::obligations::{ObligationEntityKind, ObligationReviewState, ObligationStore};
use crate::engines::obligation::{
    ObligationCandidate, ObligationEngine, ObligationEngineError, ObligationExtractionInput,
};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};

const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";
const TASK_CANDIDATE_REVIEW_SOURCE_KIND: &str = "task_candidate_review";
const TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER: &str = "local_api";
const TASK_CANDIDATE_ID_PREFIX: &str = "task_candidate:v1:";
const TASK_ID_PREFIX: &str = "task:v1:";
const TASK_CANDIDATE_EVENT_PREFIX: &str = "task_candidate_review:";
const TASK_CANDIDATE_KIND_TASK: &str = "task";
const TASK_CANDIDATE_KIND_OBLIGATION_TASK: &str = "obligation_task";
const OBLIGATION_TASK_LINK_KIND: &str = "fulfillment_task";
const OBLIGATION_CANDIDATE_METADATA_KEY: &str = "obligation_candidate";
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
const MIN_LIMIT: i64 = 1;
const REVIEW_TEXT_SNIPPET_CHARS: usize = 180;
const TITLE_PREVIEW_CHARS: usize = 120;
const OWNER_PERSONA_EXTRACTION_CONTEXT_ID: &str = "persona:owner";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateSourceKind {
    Message,
    Document,
}

impl TaskCandidateSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateKind {
    Task,
    ObligationTask,
}

impl TaskCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => TASK_CANDIDATE_KIND_TASK,
            Self::ObligationTask => TASK_CANDIDATE_KIND_OBLIGATION_TASK,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl TaskCandidateReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, TaskCandidateError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(TaskCandidateError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommand {
    pub command_id: String,
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommandResult {
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TaskCandidate {
    pub task_candidate_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub due_text: Option<String>,
    pub assignee_label: Option<String>,
    pub confidence: f64,
    pub review_state: String,
    pub evidence_excerpt: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskCandidateStore {
    pool: PgPool,
}

impl TaskCandidateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, TaskCandidateError> {
        let limit = validate_limit(limit)?;

        let message_count = self.refresh_message_candidates(limit).await?;
        let document_count = self.refresh_document_candidates(limit).await?;

        Ok(message_count + document_count)
    }

    pub async fn set_review_state(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let task_candidate_id =
            validate_non_empty("task_candidate_id", &command.task_candidate_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;
        let event_id = format!("{TASK_CANDIDATE_EVENT_PREFIX}{command_id}");
        let event = ReviewCommandEvent {
            command_id,
            task_candidate_id: task_candidate_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .into_event()?;

        EventStore::append_in_transaction(&mut transaction, &event).await?;
        self.apply_review_state_in_transaction(
            &mut transaction,
            &task_candidate_id,
            command.review_state,
            &event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;

        transaction.commit().await?;

        Ok(TaskCandidateReviewCommandResult {
            task_candidate_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), TaskCandidateError> {
        if event.event_type != TASK_CANDIDATE_REVIEW_EVENT_TYPE {
            return Err(TaskCandidateError::InvalidEventType);
        }

        let payload = ReviewEventPayload::from_payload(&event.payload)?;
        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(TaskCandidateError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;

        let mut transaction = self.pool.begin().await?;
        self.apply_review_state_in_transaction(
            &mut transaction,
            &payload.task_candidate_id,
            payload.review_state,
            &event.event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                task_candidate_id,
                source_kind,
                source_id,
                project_id,
                title,
                due_text,
                assignee_label,
                confidence,
                review_state,
                evidence_excerpt,
                generated_at,
                reviewed_at,
                updated_at
            FROM task_candidates
            ORDER BY updated_at DESC, task_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_task_candidate).collect()
    }

    async fn refresh_message_candidates(&self, limit: i64) -> Result<usize, TaskCandidateError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                body_text
            FROM communication_messages
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );

            count += self
                .refresh_message_candidate_from_text(&source_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, TaskCandidateError> {
        if message_ids.is_empty() {
            return Ok(0);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                body_text
            FROM communication_messages
            WHERE message_id = ANY($1)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .bind(message_ids.to_vec())
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );
            count += self
                .refresh_message_candidate_from_text(&source_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_message_candidate_from_text(
        &self,
        source_id: &str,
        source_text: &str,
    ) -> Result<usize, TaskCandidateError> {
        if let Some(fragment) = extract_candidate_fragment(source_text) {
            let payload = CandidatePayload {
                source_kind: TaskCandidateSourceKind::Message,
                source_id: source_id.to_owned(),
                candidate_kind: TaskCandidateKind::Task,
                candidate_metadata: json!({}),
                project_id: None,
                title: title_from_fragment(&fragment.text),
                due_text: fragment.due_text,
                assignee_label: fragment.assignee_label,
                confidence: 0.8,
                evidence_excerpt: evidence_excerpt(&fragment.text),
            };
            upsert_task_candidate(
                &self.pool,
                &payload,
                payload.task_candidate_id(),
                TaskCandidateReviewState::Suggested,
            )
            .await?;
            return Ok(1);
        }

        let input = ObligationExtractionInput::communication(
            source_id,
            source_text,
            ObligationEntityKind::Persona,
            OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
        );
        let extraction = ObligationEngine::detect_candidates(&input)?;

        let mut count = 0usize;
        for obligation_candidate in extraction.obligations {
            let payload = task_candidate_payload_from_obligation(
                TaskCandidateSourceKind::Message,
                source_id,
                &obligation_candidate,
            );
            upsert_task_candidate(
                &self.pool,
                &payload,
                payload.task_candidate_id(),
                TaskCandidateReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }

    async fn refresh_document_candidates(&self, limit: i64) -> Result<usize, TaskCandidateError> {
        let rows = sqlx::query(
            r#"
            SELECT
                document_id,
                title,
                extracted_text
            FROM documents
            ORDER BY imported_at DESC, document_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("document_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("title")?,
                row.try_get::<String, _>("extracted_text")?,
            );

            if let Some(fragment) = extract_candidate_fragment(&source_text) {
                let payload = CandidatePayload {
                    source_kind: TaskCandidateSourceKind::Document,
                    source_id,
                    candidate_kind: TaskCandidateKind::Task,
                    candidate_metadata: json!({}),
                    project_id: None,
                    title: title_from_fragment(&fragment.text),
                    due_text: fragment.due_text,
                    assignee_label: fragment.assignee_label,
                    confidence: 0.7,
                    evidence_excerpt: evidence_excerpt(&fragment.text),
                };
                upsert_task_candidate(
                    &self.pool,
                    &payload,
                    payload.task_candidate_id(),
                    TaskCandidateReviewState::Suggested,
                )
                .await?;
                count += 1;
            } else {
                let input = ObligationExtractionInput::document(
                    &source_id,
                    &source_text,
                    ObligationEntityKind::Persona,
                    OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
                );
                let extraction = ObligationEngine::detect_candidates(&input)?;

                for obligation_candidate in extraction.obligations {
                    let payload = task_candidate_payload_from_obligation(
                        TaskCandidateSourceKind::Document,
                        &source_id,
                        &obligation_candidate,
                    );
                    upsert_task_candidate(
                        &self.pool,
                        &payload,
                        payload.task_candidate_id(),
                        TaskCandidateReviewState::Suggested,
                    )
                    .await?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    async fn apply_review_state_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        task_candidate_id: &str,
        review_state: TaskCandidateReviewState,
        event_id: &str,
        actor_id: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), TaskCandidateError> {
        let candidate = row_task_candidate(transaction, task_candidate_id).await?;

        match review_state {
            TaskCandidateReviewState::UserConfirmed => {
                self.upsert_task_in_transaction(
                    transaction,
                    task_candidate_id,
                    &candidate,
                    event_id,
                    actor_id,
                    reviewed_at,
                )
                .await?;
                self.sync_obligation_candidate_review_state_in_transaction(
                    transaction,
                    task_candidate_id,
                    &candidate,
                    review_state,
                )
                .await?;

                sqlx::query(
                    r#"
                    UPDATE task_candidates
                    SET
                        review_state = $1,
                        event_id = $2,
                        actor_id = $3,
                        reviewed_at = $4,
                        updated_at = now()
                    WHERE task_candidate_id = $5
                    "#,
                )
                .bind(review_state.as_str())
                .bind(event_id)
                .bind(actor_id)
                .bind(reviewed_at)
                .bind(task_candidate_id)
                .execute(&mut **transaction)
                .await?;
            }
            TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    UPDATE task_candidates
                    SET
                        review_state = $1,
                        event_id = $2,
                        actor_id = $3,
                        reviewed_at = $4,
                        updated_at = now()
                    WHERE task_candidate_id = $5
                    "#,
                )
                .bind(review_state.as_str())
                .bind(event_id)
                .bind(actor_id)
                .bind(reviewed_at)
                .bind(task_candidate_id)
                .execute(&mut **transaction)
                .await?;

                sqlx::query("DELETE FROM tasks WHERE task_candidate_id = $1")
                    .bind(task_candidate_id)
                    .execute(&mut **transaction)
                    .await?;

                self.sync_obligation_candidate_review_state_in_transaction(
                    transaction,
                    task_candidate_id,
                    &candidate,
                    review_state,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn sync_obligation_candidate_review_state_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        task_candidate_id: &str,
        candidate: &StoredCandidateRow,
        review_state: TaskCandidateReviewState,
    ) -> Result<(), TaskCandidateError> {
        if candidate.candidate_kind != TASK_CANDIDATE_KIND_OBLIGATION_TASK {
            return Ok(());
        }

        let mut obligation_candidate = obligation_candidate_from_metadata(candidate)?;
        obligation_candidate.review_state =
            obligation_review_state_from_task_candidate(review_state);
        let (obligation, evidence) = obligation_candidate.to_obligation_draft();
        let stored_obligation = ObligationStore::upsert_with_evidence_in_transaction(
            transaction,
            &obligation,
            &[evidence],
        )
        .await?;

        if review_state != TaskCandidateReviewState::UserConfirmed {
            return Ok(());
        }

        sqlx::query(
            r#"
            INSERT INTO obligation_task_links (
                obligation_id,
                task_id,
                link_kind
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (obligation_id, task_id, link_kind) DO NOTHING
            "#,
        )
        .bind(&stored_obligation.obligation_id)
        .bind(task_id_from_candidate(task_candidate_id))
        .bind(OBLIGATION_TASK_LINK_KIND)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn upsert_task_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        task_candidate_id: &str,
        candidate: &StoredCandidateRow,
        event_id: &str,
        actor_id: &str,
        _reviewed_at: DateTime<Utc>,
    ) -> Result<(), TaskCandidateError> {
        sqlx::query(
            r#"
            INSERT INTO tasks (
                task_id,
                task_candidate_id,
                title,
                source_kind,
                source_id,
                project_id,
                status,
                created_from_event_id,
                created_by_actor_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $8)
            ON CONFLICT (task_candidate_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                source_kind = EXCLUDED.source_kind,
                source_id = EXCLUDED.source_id,
                project_id = EXCLUDED.project_id,
                status = EXCLUDED.status,
                created_from_event_id = EXCLUDED.created_from_event_id,
                created_by_actor_id = EXCLUDED.created_by_actor_id,
                updated_at = now()
            "#,
        )
        .bind(task_id_from_candidate(task_candidate_id))
        .bind(task_candidate_id)
        .bind(&candidate.title)
        .bind(&candidate.source_kind)
        .bind(&candidate.source_id)
        .bind(&candidate.project_id)
        .bind(event_id)
        .bind(actor_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
struct CandidatePayload {
    source_kind: TaskCandidateSourceKind,
    source_id: String,
    candidate_kind: TaskCandidateKind,
    candidate_metadata: Value,
    project_id: Option<String>,
    title: String,
    due_text: Option<String>,
    assignee_label: Option<String>,
    confidence: f64,
    evidence_excerpt: String,
}

impl CandidatePayload {
    fn task_candidate_id(&self) -> String {
        task_candidate_id_from_source(self.source_kind.as_str(), &self.source_id, &self.title)
    }
}

#[derive(Debug)]
struct StoredCandidateRow {
    source_kind: String,
    source_id: String,
    candidate_kind: String,
    candidate_metadata: Value,
    project_id: Option<String>,
    title: String,
}

struct ReviewCommandEvent {
    command_id: String,
    task_candidate_id: String,
    review_state: TaskCandidateReviewState,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    fn into_event(self) -> Result<NewEventEnvelope, TaskCandidateError> {
        let event_id = self.event_id.clone();
        Ok(NewEventEnvelope::builder(
            event_id,
            TASK_CANDIDATE_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": TASK_CANDIDATE_REVIEW_SOURCE_KIND,
                "provider": TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "task_candidate_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "task_candidate_id": self.task_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
struct ReviewEventPayload {
    task_candidate_id: String,
    review_state: TaskCandidateReviewState,
}

impl ReviewEventPayload {
    fn from_payload(payload: &Value) -> Result<Self, TaskCandidateError> {
        let payload = as_object(payload)?;
        Ok(Self {
            task_candidate_id: required_payload_string(payload, "task_candidate_id")?,
            review_state: TaskCandidateReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

#[derive(Debug)]
struct CandidateFragment {
    text: String,
    due_text: Option<String>,
    assignee_label: Option<String>,
}

fn extract_candidate_fragment(text: &str) -> Option<CandidateFragment> {
    let text_lower = text.to_lowercase();
    if !(text_lower.contains("action:")
        || text_lower.contains("please ")
        || text_lower.contains("follow up")
        || text_lower.contains("next step"))
    {
        return None;
    }

    let selected = text
        .lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("action:")
                || lower.contains("please ")
                || lower.contains("follow up")
                || lower.contains("next step")
        })
        .unwrap_or(text);

    let due_text = text.lines().find_map(parse_due_text);
    let assignee_label = text.lines().find_map(parse_assignee_label);

    Some(CandidateFragment {
        text: selected.to_owned(),
        due_text,
        assignee_label,
    })
}

fn parse_due_text(line: &str) -> Option<String> {
    let normalized = line.trim().to_lowercase();
    if !normalized.starts_with("due") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let due = right.trim();
        (!due.is_empty()).then_some(due.to_owned())
    })
}

fn parse_assignee_label(line: &str) -> Option<String> {
    let normalized = line.to_lowercase();
    if !normalized.starts_with("assignee") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let assignee = right.trim();
        (!assignee.is_empty()).then_some(assignee.to_owned())
    })
}

fn title_from_fragment(value: &str) -> String {
    text_preview(value, TITLE_PREVIEW_CHARS)
}

fn evidence_excerpt(value: &str) -> String {
    text_preview(value, REVIEW_TEXT_SNIPPET_CHARS)
}

fn task_candidate_payload_from_obligation(
    source_kind: TaskCandidateSourceKind,
    source_id: &str,
    candidate: &ObligationCandidate,
) -> CandidatePayload {
    CandidatePayload {
        source_kind,
        source_id: source_id.to_owned(),
        candidate_kind: TaskCandidateKind::ObligationTask,
        candidate_metadata: json!({
            "engine": "obligation",
            OBLIGATION_CANDIDATE_METADATA_KEY: candidate,
        }),
        project_id: None,
        title: title_from_fragment(&candidate.statement),
        due_text: candidate.due_text.clone(),
        assignee_label: None,
        confidence: (candidate.confidence - 0.08).max(0.0),
        evidence_excerpt: evidence_excerpt(&candidate.quote),
    }
}

fn obligation_candidate_from_metadata(
    candidate: &StoredCandidateRow,
) -> Result<ObligationCandidate, TaskCandidateError> {
    let value = candidate
        .candidate_metadata
        .get(OBLIGATION_CANDIDATE_METADATA_KEY)
        .cloned()
        .ok_or_else(|| {
            TaskCandidateError::InvalidCandidateMetadata(
                OBLIGATION_CANDIDATE_METADATA_KEY.to_owned(),
            )
        })?;

    Ok(serde_json::from_value(value)?)
}

fn obligation_review_state_from_task_candidate(
    review_state: TaskCandidateReviewState,
) -> ObligationReviewState {
    match review_state {
        TaskCandidateReviewState::Suggested => ObligationReviewState::Suggested,
        TaskCandidateReviewState::UserConfirmed => ObligationReviewState::UserConfirmed,
        TaskCandidateReviewState::UserRejected => ObligationReviewState::UserRejected,
    }
}

async fn upsert_task_candidate(
    pool: &PgPool,
    payload: &CandidatePayload,
    task_candidate_id: String,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    let update_result = sqlx::query(
        r#"
        UPDATE task_candidates
        SET
            source_kind = $2,
            source_id = $3,
            candidate_kind = $4,
            candidate_metadata = $5,
            project_id = COALESCE($6, project_id),
            title = $7,
            due_text = COALESCE($8, due_text),
            assignee_label = COALESCE($9, assignee_label),
            confidence = $10,
            review_state = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN review_state
                ELSE $11
            END,
            evidence_excerpt = $12,
            event_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        WHERE task_candidate_id = $1
           OR (source_kind = $2 AND source_id = $3 AND lower(title) = lower($7))
        "#,
    )
    .bind(&task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    if update_result.rows_affected() > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
            task_candidate_id,
            source_kind,
            source_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            review_state,
            evidence_excerpt,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NULL, NULL, NULL)
        ON CONFLICT (source_kind, source_id, lower(title))
        DO UPDATE SET
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            candidate_kind = EXCLUDED.candidate_kind,
            candidate_metadata = EXCLUDED.candidate_metadata,
            project_id = COALESCE(EXCLUDED.project_id, task_candidates.project_id),
            title = EXCLUDED.title,
            due_text = COALESCE(EXCLUDED.due_text, task_candidates.due_text),
            assignee_label = COALESCE(EXCLUDED.assignee_label, task_candidates.assignee_label),
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            evidence_excerpt = EXCLUDED.evidence_excerpt,
            event_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.actor_id
                ELSE NULL
                END,
            reviewed_at = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    Ok(())
}

async fn row_task_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
) -> Result<StoredCandidateRow, TaskCandidateError> {
    let row = sqlx::query(
        r#"
        SELECT
            source_kind,
            source_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title
        FROM task_candidates
        WHERE task_candidate_id = $1
        FOR UPDATE
        "#,
    )
    .bind(task_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(TaskCandidateError::TaskCandidateNotFound)?;

    Ok(StoredCandidateRow {
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        candidate_metadata: row.try_get("candidate_metadata")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
    })
}

fn row_to_task_candidate(row: sqlx::postgres::PgRow) -> Result<TaskCandidate, TaskCandidateError> {
    Ok(TaskCandidate {
        task_candidate_id: row.try_get("task_candidate_id")?,
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
        due_text: row.try_get("due_text")?,
        assignee_label: row.try_get("assignee_label")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get::<String, _>("review_state")?,
        evidence_excerpt: row.try_get("evidence_excerpt")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, TaskCandidateError> {
    value
        .as_object()
        .ok_or_else(|| TaskCandidateError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, TaskCandidateError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| TaskCandidateError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| TaskCandidateError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

fn task_candidate_id_from_source(source_kind: &str, source_id: &str, title: &str) -> String {
    let title_hash = fnv1a64_hex(title);
    format!("{TASK_CANDIDATE_ID_PREFIX}{source_kind}:{source_id}:{title_hash}")
}

fn task_id_from_candidate(task_candidate_id: &str) -> String {
    format!("{TASK_ID_PREFIX}{}", fnv1a64_hex(task_candidate_id))
}

fn fnv1a64_hex(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{hash:016x}")
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, TaskCandidateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskCandidateError::EmptyField(field.to_owned()));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, TaskCandidateError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(TaskCandidateError::InvalidLimit);
    }

    Ok(limit)
}

fn validate_optional_limit(limit: Option<i64>) -> Result<i64, TaskCandidateError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}

fn text_preview(value: &str, max_chars: usize) -> String {
    let preview = value.trim().chars().take(max_chars).collect::<String>();
    if value.trim().chars().count() > max_chars {
        format!("{preview}...")
    } else {
        preview
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("task candidate was not found")]
    TaskCandidateNotFound,

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("candidate metadata is missing or invalid: {0}")]
    InvalidCandidateMetadata(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    ObligationEngine(#[from] ObligationEngineError),

    #[error(transparent)]
    ObligationStore(#[from] crate::domains::obligations::ObligationStoreError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
