use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{Decision, DecisionReviewState, DecisionStore, DecisionStoreError};
use crate::domains::obligations::{
    Obligation, ObligationReviewState, ObligationStore, ObligationStoreError,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, sync_decision_review_state_with_observation,
    sync_obligation_review_state_with_observation,
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
