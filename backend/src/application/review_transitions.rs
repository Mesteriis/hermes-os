use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{Decision, DecisionReviewState, DecisionStore, DecisionStoreError};
use crate::domains::obligations::{
    Obligation, ObligationReviewState, ObligationStore, ObligationStoreError,
};
use crate::domains::relationships::{
    Relationship, RelationshipReviewState, RelationshipStore, RelationshipStoreError,
};
use crate::domains::tasks::candidates::{
    StoredCandidateRow, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewService, TaskCandidateReviewServiceError, TaskCandidateReviewState,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, sync_decision_review_state_with_observation,
    sync_obligation_review_state_with_observation, sync_relationship_review_state_with_observation,
    sync_task_candidate_review_state_in_transaction,
};

#[derive(Clone)]
pub struct DecisionReviewApplicationService {
    pool: PgPool,
}

impl DecisionReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "decision_id": decision_id,
                        "review_state": review_state.as_str(),
                        "operation": "decision_review",
                        "actor_id": "hermes-frontend",
                    }),
                    format!("decision://{decision_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "decision_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let decision = DecisionStore::new(self.pool.clone())
            .set_review_state_with_observation(
                decision_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "decision_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_decision_review_state_with_observation(
            &self.pool,
            &decision,
            &observation.observation_id,
        )
        .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct ObligationReviewApplicationService {
    pool: PgPool,
}

impl ObligationReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "obligation_id": obligation_id,
                        "review_state": review_state.as_str(),
                        "operation": "obligation_review",
                        "actor_id": "hermes-frontend",
                    }),
                    format!("obligation://{obligation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "obligation_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let obligation = ObligationStore::new(self.pool.clone())
            .set_review_state_with_observation(
                obligation_id,
                review_state,
                Some(&observation.observation_id),
                None,
            )
            .await?;

        sync_obligation_review_state_with_observation(
            &self.pool,
            &obligation,
            &observation.observation_id,
        )
        .await?;

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct RelationshipReviewApplicationService {
    pool: PgPool,
}

impl RelationshipReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "relationship_id": relationship_id,
                        "review_state": review_state.as_str(),
                        "operation": "relationship_review",
                        "actor_id": "hermes-frontend",
                    }),
                    format!("relationship://{relationship_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let relationship = RelationshipStore::new(self.pool.clone())
            .set_review_state_with_observation(
                relationship_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_relationship_review_state_with_observation(
            &self.pool,
            &relationship,
            &observation.observation_id,
        )
        .await?;

        Ok(relationship)
    }
}

#[derive(Debug, Error)]
pub enum RelationshipReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct TaskCandidateReviewApplicationService {
    pool: PgPool,
}

impl TaskCandidateReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewApplicationError> {
        let result = TaskCandidateReviewService::new(self.pool.clone())
            .review_manual(command)
            .await?;

        let mut transaction = self.pool.begin().await?;
        let candidate_row = sqlx::query(
            r#"
            SELECT
                source_kind,
                source_id,
                observation_id,
                candidate_kind,
                candidate_metadata,
                project_id,
                title,
                due_text,
                assignee_label,
                confidence::float8 AS confidence,
                evidence_excerpt
            FROM task_candidates
            WHERE task_candidate_id = $1
            FOR UPDATE
            "#,
        )
        .bind(&command.task_candidate_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(TaskCandidateReviewApplicationError::TaskCandidateNotFound)?;
        let candidate = StoredCandidateRow {
            source_kind: sqlx::Row::try_get(&candidate_row, "source_kind")?,
            source_id: sqlx::Row::try_get(&candidate_row, "source_id")?,
            observation_id: sqlx::Row::try_get(&candidate_row, "observation_id")?,
            candidate_kind: sqlx::Row::try_get(&candidate_row, "candidate_kind")?,
            candidate_metadata: sqlx::Row::try_get(&candidate_row, "candidate_metadata")?,
            project_id: sqlx::Row::try_get(&candidate_row, "project_id")?,
            title: sqlx::Row::try_get(&candidate_row, "title")?,
            due_text: sqlx::Row::try_get(&candidate_row, "due_text")?,
            assignee_label: sqlx::Row::try_get(&candidate_row, "assignee_label")?,
            confidence: sqlx::Row::try_get(&candidate_row, "confidence")?,
            evidence_excerpt: sqlx::Row::try_get(&candidate_row, "evidence_excerpt")?,
        };
        sync_task_candidate_review_state_in_transaction(
            &mut transaction,
            &command.task_candidate_id,
            &candidate,
            command.review_state,
        )
        .await?;
        transaction.commit().await?;

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateReviewApplicationError {
    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateReviewServiceError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error("task candidate was not found")]
    TaskCandidateNotFound,
}
