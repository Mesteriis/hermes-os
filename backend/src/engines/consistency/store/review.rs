use sqlx::postgres::PgPool;

use super::super::errors::ConsistencyError;
use super::super::models::{ContradictionObservation, ContradictionReviewState};
use super::super::rows::row_to_observation;
use super::super::validation::validate_non_empty;

pub(super) async fn set_review_state(
    pool: &PgPool,
    observation_id: &str,
    review_state: ContradictionReviewState,
    reviewed_by: &str,
    resolution: Option<&str>,
) -> Result<ContradictionObservation, ConsistencyError> {
    validate_non_empty("observation_id", observation_id)?;
    validate_non_empty("reviewed_by", reviewed_by)?;
    let row = sqlx::query(
        r#"
        UPDATE contradiction_observations
        SET review_state = $2,
            reviewed_by = $3,
            reviewed_at = now(),
            resolution = $4,
            updated_at = now()
        WHERE observation_id = $1
        RETURNING
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        "#,
    )
    .bind(observation_id)
    .bind(review_state.as_str())
    .bind(reviewed_by)
    .bind(resolution)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Err(ConsistencyError::ObservationNotFound(
            observation_id.to_owned(),
        ));
    };

    row_to_observation(row)
}
