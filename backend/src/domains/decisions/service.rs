use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{Decision, DecisionReviewState, DecisionStore, DecisionStoreError};

#[derive(Clone)]
pub struct DecisionCommandService {
    pool: PgPool,
}

impl DecisionCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionCommandServiceError> {
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
                    "captured_by": "decisions_service.review_manual",
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
                    "captured_by": "decisions_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
}
