use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonaIdentityError;
use super::models::PersonaIdentityCandidate;

pub(super) fn row_to_persona_identity_candidate(
    row: PgRow,
) -> Result<PersonaIdentityCandidate, PersonaIdentityError> {
    Ok(PersonaIdentityCandidate {
        identity_candidate_id: row.try_get("identity_candidate_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        left_persona_id: row.try_get("left_persona_id")?,
        right_persona_id: row.try_get("right_persona_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get("review_state")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
