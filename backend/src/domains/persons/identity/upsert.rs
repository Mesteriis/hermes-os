use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use super::errors::PersonIdentityError;
use super::models::{PersonIdentityCandidatePayload, PersonIdentityReviewState};

pub(super) async fn upsert_candidate(
    pool: &PgPool,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        INSERT INTO person_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_person_id = EXCLUDED.left_person_id,
            right_person_id = EXCLUDED.right_person_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_person_id)
    .bind(&payload.right_person_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) async fn upsert_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        INSERT INTO person_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_person_id = EXCLUDED.left_person_id,
            right_person_id = EXCLUDED.right_person_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_person_id)
    .bind(&payload.right_person_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
