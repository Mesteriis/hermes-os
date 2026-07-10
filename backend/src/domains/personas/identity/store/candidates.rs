use super::super::errors::PersonaIdentityError;
use super::super::models::{
    PersonaIdentityCandidateKind, PersonaIdentityCandidatePayload, PersonaIdentityReviewState,
};
use super::super::upsert::upsert_candidate;
use super::super::validation::validate_limit;
use super::PersonaIdentityReviewStore;
use super::name_merge_candidates::refresh_name_merge_candidates;
use super::split_candidates::refresh_split_candidates;
use sqlx::Row;

impl PersonaIdentityReviewStore {
    pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, PersonaIdentityError> {
        let limit = validate_limit(limit)?;
        let merge_count = refresh_name_merge_candidates(self.pool(), limit).await?;
        let split_count = refresh_split_candidates(self.pool(), limit).await?;

        Ok(merge_count + split_count)
    }

    pub async fn suggest_attach_email_candidates(
        &self,
        display_name: &str,
        email_address: &str,
        evidence_summary: &str,
        confidence: f64,
        limit: i64,
    ) -> Result<usize, PersonaIdentityError> {
        let limit = validate_limit(limit)?;
        let normalized_display_name = display_name.trim().to_ascii_lowercase();
        let normalized_email = email_address.trim().to_ascii_lowercase();
        if normalized_display_name.is_empty()
            || normalized_email.is_empty()
            || !normalized_email.contains('@')
        {
            return Ok(0);
        }

        let rows = sqlx::query(
            r#"
            SELECT person.person_id
            FROM personas person
            WHERE lower(trim(person.display_name)) = $1
              AND position('@' in lower(trim(person.display_name))) = 0
              AND NOT EXISTS (
                    SELECT 1
                    FROM persona_identities identity_trace
                    WHERE identity_trace.person_id = person.person_id
                      AND identity_trace.identity_type = 'email'
                      AND lower(trim(identity_trace.identity_value)) = $2
                      AND identity_trace.status = 'active'
              )
            ORDER BY person.person_id ASC
            LIMIT $3
            "#,
        )
        .bind(&normalized_display_name)
        .bind(&normalized_email)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        let mut count = 0usize;
        for row in rows {
            let candidate = PersonaIdentityCandidatePayload {
                candidate_kind: PersonaIdentityCandidateKind::AttachEmailAddress,
                left_persona_id: row.try_get("person_id")?,
                right_persona_id: None,
                email_address: Some(normalized_email.clone()),
                evidence_summary: evidence_summary.trim().to_owned(),
                confidence,
            };
            upsert_candidate(
                self.pool(),
                &candidate,
                candidate.identity_candidate_id(),
                PersonaIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }
}
