use chrono::{DateTime, Utc};
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::super::errors::PersonaIdentityError;
use super::super::models::PersonaIdentityReviewState;

pub(super) async fn apply_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<(), PersonaIdentityError> {
    match review_state {
        PersonaIdentityReviewState::Suggested => {
            sqlx::query(
                r#"
                UPDATE persona_identity_candidates
                SET
                    review_state = $1,
                    event_id = NULL,
                    actor_id = NULL,
                    reviewed_at = NULL,
                    updated_at = now()
                WHERE identity_candidate_id = $2
                "#,
            )
            .bind(review_state.as_str())
            .bind(identity_candidate_id)
            .execute(&mut **transaction)
            .await?;
        }
        PersonaIdentityReviewState::UserConfirmed | PersonaIdentityReviewState::UserRejected => {
            sqlx::query(
                r#"
                UPDATE persona_identity_candidates
                SET
                    review_state = $1,
                    event_id = $2,
                    actor_id = $3,
                    reviewed_at = $4,
                    updated_at = now()
                WHERE identity_candidate_id = $5
                "#,
            )
            .bind(review_state.as_str())
            .bind(event_id)
            .bind(actor_id)
            .bind(reviewed_at)
            .bind(identity_candidate_id)
            .execute(&mut **transaction)
            .await?;
        }
    }

    Ok(())
}

pub(super) async fn ensure_candidate_exists(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
) -> Result<(), PersonaIdentityError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM persona_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&mut **transaction)
    .await?;

    if !exists {
        return Err(PersonaIdentityError::IdentityCandidateNotFound);
    }

    Ok(())
}
