use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    ConsistencyError, ContradictionObservation, ContradictionObservationStore,
    ContradictionReviewState,
};

#[derive(Clone)]
pub struct ContradictionReviewService {
    pool: PgPool,
}

impl ContradictionReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ContradictionReviewServiceError> {
        let review_observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "contradiction_observation_id": observation_id,
                        "review_state": review_state.as_str(),
                        "resolution": resolution,
                        "operation": "contradiction_review",
                        "actor_id": "hermes-frontend",
                    }),
                    format!("contradiction://{observation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "consistency.review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(ContradictionObservationStore::new(self.pool.clone())
            .set_review_state_with_observation(
                observation_id,
                review_state,
                "hermes-frontend",
                resolution,
                Some(&review_observation.observation_id),
                None,
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum ContradictionReviewServiceError {
    #[error(transparent)]
    Consistency(#[from] ConsistencyError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
