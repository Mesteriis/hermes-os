use super::super::errors::PersonaIdentityError;
use super::super::models::{PersonaIdentityCandidate, PersonaIdentityDetail};
use super::super::rows::row_to_persona_identity_candidate;
use super::super::validation::{validate_non_empty, validate_optional_limit};
use super::PersonaIdentityReviewStore;

impl PersonaIdentityReviewStore {
    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<PersonaIdentityCandidate>, PersonaIdentityError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM persona_identity_candidates
            ORDER BY updated_at DESC, identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(row_to_persona_identity_candidate)
            .collect()
    }

    pub async fn persona_identity(
        &self,
        person_id: &str,
    ) -> Result<PersonaIdentityDetail, PersonaIdentityError> {
        let person_id = validate_non_empty("person_id", person_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM persona_identity_candidates merge
            WHERE (merge.left_person_id = $1 OR merge.right_person_id = $1)
              AND merge.candidate_kind IN ('merge_personas', 'merge_persons')
              AND merge.review_state = 'user_confirmed'
              AND NOT EXISTS (
                  SELECT 1
                  FROM persona_identity_candidates split
                  WHERE split.candidate_kind IN ('split_persona', 'split_person')
                    AND split.review_state = 'user_confirmed'
                    AND LEAST(split.left_person_id, split.right_person_id) =
                        LEAST(merge.left_person_id, merge.right_person_id)
                    AND GREATEST(split.left_person_id, split.right_person_id) =
                        GREATEST(merge.left_person_id, merge.right_person_id)
              )
            ORDER BY updated_at DESC, identity_candidate_id
            "#,
        )
        .bind(&person_id)
        .fetch_all(self.pool())
        .await?;

        let items = rows
            .into_iter()
            .map(row_to_persona_identity_candidate)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PersonaIdentityDetail { items })
    }
}
