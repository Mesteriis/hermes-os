use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};

use super::super::errors::PersonaIdentityError;
use super::super::models::{
    PersonaIdentityCandidateKind, PersonaIdentityCandidatePayload, PersonaIdentityReviewState,
};
use super::super::upsert::{upsert_candidate, upsert_candidate_in_transaction};

pub(super) async fn refresh_split_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, PersonaIdentityError> {
    let rows = sqlx::query(
        r#"
        SELECT
            merge.left_person_id,
            merge.right_person_id
        FROM persona_identity_candidates merge
        WHERE merge.candidate_kind IN ('merge_personas', 'merge_persons')
          AND merge.review_state = 'user_confirmed'
          AND merge.right_person_id IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM persona_identity_candidates split
              WHERE split.candidate_kind IN ('split_persona', 'split_person')
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
            PersonaIdentityReviewState::Suggested,
        )
        .await?;
        count += 1;
    }

    Ok(count)
}

pub(super) async fn materialize_split_candidate_for_confirmed_merge_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
) -> Result<(), PersonaIdentityError> {
    if review_state != PersonaIdentityReviewState::UserConfirmed {
        return Ok(());
    }

    let row = sqlx::query(
        r#"
        SELECT candidate_kind, left_person_id, right_person_id
        FROM persona_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&mut **transaction)
    .await?;

    let candidate_kind = row.try_get::<String, _>("candidate_kind")?;
    if !matches!(candidate_kind.as_str(), "merge_personas" | "merge_persons") {
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
        PersonaIdentityReviewState::Suggested,
    )
    .await
}

fn split_candidate_payload(left: String, right: String) -> PersonaIdentityCandidatePayload {
    PersonaIdentityCandidatePayload {
        candidate_kind: PersonaIdentityCandidateKind::SplitPersona,
        left_persona_id: left.clone(),
        right_persona_id: Some(right.clone()),
        email_address: None,
        evidence_summary: format!("Previously confirmed merge can be split: {left} and {right}"),
        confidence: 1.0,
    }
}
