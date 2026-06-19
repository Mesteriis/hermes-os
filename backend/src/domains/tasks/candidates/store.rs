mod list;
mod obligation_sync;
mod refresh;
mod review;
mod task_activation;

use sqlx::postgres::PgPool;

use super::errors::TaskCandidateError;
use super::models::{TaskCandidate, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult};
use crate::platform::events::EventEnvelope;
use serde_json::Value;

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
        refresh::refresh_deterministic_candidates(&self.pool, limit).await
    }

    pub async fn set_review_state(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
        review::set_review_state(&self.pool, command).await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &TaskCandidateReviewCommand,
        observation_id: &str,
        metadata: Value,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
        review::set_review_state_with_observation(
            &self.pool,
            command,
            Some(observation_id),
            Some(metadata),
        )
        .await
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), TaskCandidateError> {
        review::apply_review_event(&self.pool, event).await
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
        list::list_candidates(&self.pool, limit).await
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, TaskCandidateError> {
        refresh::refresh_message_candidates_for_ids(&self.pool, message_ids).await
    }
}
