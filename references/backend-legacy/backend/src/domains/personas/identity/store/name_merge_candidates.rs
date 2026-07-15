use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::PersonaIdentityError;
use super::super::models::{
    PersonaIdentityCandidateKind, PersonaIdentityCandidatePayload, PersonaIdentityReviewState,
};
use super::super::upsert::upsert_candidate;

pub(super) async fn refresh_name_merge_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, PersonaIdentityError> {
    let rows = sqlx::query(
        r#"
        SELECT
            c1.persona_id AS left_persona_id,
            c2.persona_id AS right_persona_id,
            lower(trim(c1.display_name)) AS normalized_display_name
        FROM personas c1
        JOIN personas c2
            ON c1.persona_id < c2.persona_id
           AND lower(trim(c1.display_name)) = lower(trim(c2.display_name))
        WHERE position('@' in lower(trim(c1.display_name))) = 0
          AND position('@' in lower(trim(c2.display_name))) = 0
        ORDER BY
            lower(trim(c1.display_name)),
            c1.persona_id,
            c2.persona_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let left = row.try_get::<String, _>("left_persona_id")?;
        let right = row.try_get::<String, _>("right_persona_id")?;
        let candidate = PersonaIdentityCandidatePayload {
            candidate_kind: PersonaIdentityCandidateKind::MergePersonas,
            left_persona_id: left,
            right_persona_id: Some(right),
            email_address: None,
            evidence_summary: format!(
                "Same normalized display name: {}",
                row.try_get::<String, _>("normalized_display_name")?
            ),
            confidence: 0.72,
        };
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
