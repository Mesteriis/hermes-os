use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use hermes_observations_api::models::{NewObservation, ObservationOriginKind};
use hermes_observations_postgres::errors::ObservationStoreError;
use hermes_observations_postgres::store::ObservationStore;

use super::{Obligation, ObligationReviewState, ObligationStore, ObligationStoreError};

#[derive(Clone)]
pub struct ObligationCommandService {
    pool: PgPool,
}

impl ObligationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationCommandServiceError> {
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
                    "captured_by": "obligations_service.review_manual",
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

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
}
