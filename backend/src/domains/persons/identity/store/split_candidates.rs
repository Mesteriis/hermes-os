use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};

use super::super::errors::PersonIdentityError;
use super::super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use super::super::upsert::{upsert_candidate, upsert_candidate_in_transaction};

pub(super) async fn refresh_split_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, PersonIdentityError> {
    let rows = sqlx::query(
        r#"
        SELECT
            merge.left_person_id,
            merge.right_person_id
        FROM person_identity_candidates merge
        WHERE merge.candidate_kind = 'merge_persons'
          AND merge.review_state = 'user_confirmed'
          AND merge.right_person_id IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM person_identity_candidates split
              WHERE split.candidate_kind = 'split_person'
                AND split.left_person_id = merge.left_person_id
                AND split.right_person_id = merge.right_person_id
          )
        ORDER BY merge.updated_at DESC, merge.identity_candidate_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let left = row.try_get::<String, _>("left_person_id")?;
        let right = row.try_get::<String, _>("right_person_id")?;
        let candidate = split_candidate_payload(left, right);
        upsert_candidate(
            pool,
            &candidate,
            candidate.identity_candidate_id(),
            PersonIdentityReviewState::Suggested,
        )
        .await?;
        count += 1;
    }

    Ok(count)
}

pub(super) async fn materialize_split_candidate_for_confirmed_merge_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    if review_state != PersonIdentityReviewState::UserConfirmed {
        return Ok(());
    }

    let row = sqlx::query(
        r#"
        SELECT candidate_kind, left_person_id, right_person_id
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&mut **transaction)
    .await?;

    let candidate_kind = row.try_get::<String, _>("candidate_kind")?;
    if candidate_kind != PersonIdentityCandidateKind::MergePersons.as_str() {
        return Ok(());
    }

    let left = row.try_get::<String, _>("left_person_id")?;
    let Some(right) = row.try_get::<Option<String>, _>("right_person_id")? else {
        return Ok(());
    };
    let candidate = split_candidate_payload(left, right);
    upsert_candidate_in_transaction(
        transaction,
        &candidate,
        candidate.identity_candidate_id(),
        PersonIdentityReviewState::Suggested,
    )
    .await
}

fn split_candidate_payload(left: String, right: String) -> PersonIdentityCandidatePayload {
    PersonIdentityCandidatePayload {
        candidate_kind: PersonIdentityCandidateKind::SplitPerson,
        left_person_id: left.clone(),
        right_person_id: Some(right.clone()),
        email_address: None,
        evidence_summary: format!("Previously confirmed merge can be split: {left} and {right}"),
        confidence: 1.0,
    }
}
