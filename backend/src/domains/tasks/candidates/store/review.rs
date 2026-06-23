use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState,
};
use crate::domains::tasks::core::ObligationTaskLinkPort;
use crate::engines::obligation::ObligationCandidate;
use crate::platform::events::{EventEnvelope, EventStore};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

use super::super::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, TASK_CANDIDATE_EVENT_PREFIX,
    TASK_CANDIDATE_KIND_OBLIGATION_TASK, TASK_CANDIDATE_REVIEW_EVENT_TYPE,
};
use super::super::errors::TaskCandidateError;
use super::super::events::{ReviewCommandEvent, ReviewEventPayload};
use super::super::ids::task_id_from_candidate;
use super::super::models::{
    StoredCandidateRow, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewState,
};
use super::super::persistence::row_task_candidate;
use super::super::validation::validate_non_empty;
use super::task_activation::upsert_task_in_transaction;

pub(super) async fn set_review_state(
    pool: &PgPool,
    command: &TaskCandidateReviewCommand,
) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
    set_review_state_with_observation(pool, command, None, None).await
}

pub(super) async fn set_review_state_with_observation(
    pool: &PgPool,
    command: &TaskCandidateReviewCommand,
    observation_id: Option<&str>,
    metadata: Option<Value>,
) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
    let command_id = validate_non_empty("command_id", &command.command_id)?;
    let task_candidate_id = validate_non_empty("task_candidate_id", &command.task_candidate_id)?;
    let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

    let mut transaction = pool.begin().await?;
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
    apply_review_state_in_transaction(
        &mut transaction,
        &task_candidate_id,
        command.review_state,
        &event_id,
        &actor_id,
        event.occurred_at,
    )
    .await?;
    let metadata = match metadata {
        Some(extra) => Some(json!({
            "event_id": event_id,
            "context": extra,
        })),
        None => Some(json!({
            "event_id": event_id,
        })),
    };
    materialize_review_transition_link_in_transaction(
        &mut transaction,
        observation_id,
        "tasks",
        "task_candidate",
        &task_candidate_id,
        "review_state",
        command.review_state.as_str(),
        metadata,
    )
    .await?;

    transaction.commit().await?;

    Ok(TaskCandidateReviewCommandResult {
        task_candidate_id,
        review_state: command.review_state,
        event_id,
    })
}

pub(super) async fn apply_review_event(
    pool: &PgPool,
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

    let mut transaction = pool.begin().await?;
    apply_review_state_in_transaction(
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

async fn apply_review_state_in_transaction(
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
            upsert_task_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                event_id,
                actor_id,
            )
            .await?;
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
            sync_obligation_candidate_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                TaskCandidateReviewState::UserConfirmed,
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            sync_obligation_candidate_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                review_state,
            )
            .await?;
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
            delete_task_for_candidate(transaction, task_candidate_id).await?;
        }
    }

    Ok(())
}

async fn sync_obligation_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    if candidate.candidate_kind != TASK_CANDIDATE_KIND_OBLIGATION_TASK {
        return Ok(());
    }

    match review_state {
        TaskCandidateReviewState::UserConfirmed => {
            let observation_id = candidate.observation_id.as_deref().ok_or_else(|| {
                TaskCandidateError::ObservationRequired(task_candidate_id.to_owned())
            })?;
            let obligation_candidate =
                obligation_candidate_from_metadata(&candidate.candidate_metadata)?;
            let mut obligation = NewObligation::new(
                map_obligation_entity_kind(obligation_candidate.obligated_entity_kind),
                obligation_candidate.obligated_entity_id.clone(),
                obligation_candidate.statement.clone(),
                obligation_candidate.confidence,
                ObligationReviewState::UserConfirmed,
            )
            .metadata(json!({
                "task_candidate_id": task_candidate_id,
                "candidate_kind": TASK_CANDIDATE_KIND_OBLIGATION_TASK,
            }));
            if let (Some(kind), Some(entity_id)) = (
                obligation_candidate.beneficiary_entity_kind,
                obligation_candidate.beneficiary_entity_id.as_deref(),
            ) {
                obligation = obligation.beneficiary(map_obligation_entity_kind(kind), entity_id);
            }
            if let Some(condition) = obligation_candidate.condition.as_deref() {
                obligation = obligation.condition(condition);
            }

            let evidence = [NewObligationEvidence::observation(observation_id)
                .quote(obligation_candidate.quote.clone())
                .confidence(obligation_candidate.confidence)
                .metadata(json!({
                    "task_candidate_id": task_candidate_id,
                }))];
            let stored = ObligationReviewPort::upsert_with_evidence_in_transaction(
                transaction,
                &obligation,
                &evidence,
            )
            .await?;
            ObligationTaskLinkPort::link_fulfillment_task_in_transaction(
                transaction,
                &stored.obligation_id,
                &task_id_from_candidate(task_candidate_id),
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            let linked_obligation_ids = sqlx::query_scalar::<_, String>(
                r#"
                SELECT link.obligation_id
                FROM obligation_task_links link
                JOIN tasks task
                  ON task.task_id = link.task_id
                WHERE task.task_candidate_id = $1
                  AND link.link_kind = 'fulfillment_task'
                ORDER BY link.obligation_id
                "#,
            )
            .bind(task_candidate_id)
            .fetch_all(&mut **transaction)
            .await?;
            let obligation_review_state = match review_state {
                TaskCandidateReviewState::Suggested => ObligationReviewState::Suggested,
                TaskCandidateReviewState::UserRejected => ObligationReviewState::UserRejected,
                TaskCandidateReviewState::UserConfirmed => unreachable!(),
            };
            for obligation_id in linked_obligation_ids {
                ObligationReviewPort::set_review_state_in_transaction(
                    transaction,
                    &obligation_id,
                    obligation_review_state,
                    candidate.observation_id.as_deref(),
                    Some(json!({
                        "task_candidate_id": task_candidate_id,
                        "review_state": review_state.as_str(),
                    })),
                )
                .await?;
            }
        }
    }

    Ok(())
}

fn obligation_candidate_from_metadata(
    metadata: &Value,
) -> Result<ObligationCandidate, TaskCandidateError> {
    let candidate = metadata
        .get(OBLIGATION_CANDIDATE_METADATA_KEY)
        .cloned()
        .ok_or_else(|| {
            TaskCandidateError::InvalidCandidateMetadata(
                OBLIGATION_CANDIDATE_METADATA_KEY.to_owned(),
            )
        })?;
    Ok(serde_json::from_value(candidate)?)
}

fn map_obligation_entity_kind(
    value: crate::engines::obligation::ObligationEntityKind,
) -> ObligationEntityKind {
    match value {
        crate::engines::obligation::ObligationEntityKind::Persona => ObligationEntityKind::Persona,
        crate::engines::obligation::ObligationEntityKind::Organization => {
            ObligationEntityKind::Organization
        }
        crate::engines::obligation::ObligationEntityKind::Project => ObligationEntityKind::Project,
        crate::engines::obligation::ObligationEntityKind::Communication => {
            ObligationEntityKind::Communication
        }
        crate::engines::obligation::ObligationEntityKind::Document => {
            ObligationEntityKind::Document
        }
        crate::engines::obligation::ObligationEntityKind::Task => ObligationEntityKind::Task,
        crate::engines::obligation::ObligationEntityKind::Event => ObligationEntityKind::Event,
        crate::engines::obligation::ObligationEntityKind::Decision => {
            ObligationEntityKind::Decision
        }
        crate::engines::obligation::ObligationEntityKind::Obligation => {
            ObligationEntityKind::Obligation
        }
        crate::engines::obligation::ObligationEntityKind::Knowledge => {
            ObligationEntityKind::Knowledge
        }
    }
}

async fn update_candidate_review_state(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<(), TaskCandidateError> {
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

    Ok(())
}

async fn delete_task_for_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
) -> Result<(), TaskCandidateError> {
    sqlx::query("DELETE FROM tasks WHERE task_candidate_id = $1")
        .bind(task_candidate_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}
