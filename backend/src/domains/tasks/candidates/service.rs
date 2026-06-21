use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateStore,
};

#[derive(Clone)]
pub struct TaskCandidateReviewService {
    pool: PgPool,
}

impl TaskCandidateReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "task_candidate_id": command.task_candidate_id,
                        "command_id": command.command_id,
                        "review_state": command.review_state.as_str(),
                        "actor_id": command.actor_id,
                        "operation": "task_candidate_review",
                    }),
                    format!(
                        "task-candidate://{}/review/{}",
                        command.task_candidate_id, command.command_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "tasks.candidates_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(TaskCandidateStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                &observation.observation_id,
                json!({
                    "captured_by": "tasks.candidates_service.review_manual",
                    "operation": "review_manual",
                }),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateReviewServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),
}
