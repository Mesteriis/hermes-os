use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use super::errors::PersonIdentityError;
use super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};

pub(super) async fn upsert_candidate(
    pool: &PgPool,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    let review_state: String = sqlx::query_scalar(
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
        RETURNING review_state
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
    .fetch_one(pool)
    .await?;

    Ok(())
}

pub(super) async fn upsert_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    let review_state: String = sqlx::query_scalar(
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
        RETURNING review_state
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
    .fetch_one(&mut **transaction)
    .await?;

    Ok(())
}

pub(crate) async fn sync_identity_candidate_review_state_to_inbox_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    let _ = (transaction, identity_candidate_id, review_state);
    Ok(())
}

async fn load_identity_candidate_payload(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
) -> Result<PersonIdentityCandidatePayload, PersonIdentityError> {
    let row = sqlx::query(
        r#"
        SELECT
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence::float8 AS confidence
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?;

    let row = row.ok_or(PersonIdentityError::IdentityCandidateNotFound)?;
    let candidate_kind = match row.try_get::<String, _>("candidate_kind")?.as_str() {
        "merge_persons" => PersonIdentityCandidateKind::MergePersons,
        "attach_email_address" => PersonIdentityCandidateKind::AttachEmailAddress,
        "split_person" => PersonIdentityCandidateKind::SplitPerson,
        other => return Err(PersonIdentityError::InvalidCandidateKind(other.to_owned())),
    };

    Ok(PersonIdentityCandidatePayload {
        candidate_kind,
        left_person_id: row.try_get("left_person_id")?,
        right_person_id: row.try_get("right_person_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
    })
}
