use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};

use crate::domains::obligations::{ObligationEntityKind, ObligationStore};
use crate::engines::obligation::{ObligationEngine, ObligationExtractionInput};
use crate::platform::events::{EventEnvelope, EventStore};

use super::constants::{
    OBLIGATION_TASK_LINK_KIND, OWNER_PERSONA_EXTRACTION_CONTEXT_ID, TASK_CANDIDATE_EVENT_PREFIX,
    TASK_CANDIDATE_KIND_OBLIGATION_TASK, TASK_CANDIDATE_REVIEW_EVENT_TYPE,
};
use super::errors::TaskCandidateError;
use super::events::{ReviewCommandEvent, ReviewEventPayload};
use super::extraction::{
    evidence_excerpt, extract_candidate_fragment, obligation_candidate_from_metadata,
    obligation_review_state_from_task_candidate, task_candidate_payload_from_obligation,
    title_from_fragment,
};
use super::ids::task_id_from_candidate;
use super::models::{
    CandidatePayload, StoredCandidateRow, TaskCandidate, TaskCandidateKind,
    TaskCandidateReviewCommand, TaskCandidateReviewCommandResult, TaskCandidateReviewState,
    TaskCandidateSourceKind,
};
use super::persistence::{row_task_candidate, row_to_task_candidate, upsert_task_candidate};
use super::validation::{validate_limit, validate_non_empty, validate_optional_limit};

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
