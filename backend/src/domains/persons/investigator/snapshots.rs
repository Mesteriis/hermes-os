use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::InvestigatorError;
use super::models::{DossierReviewState, DossierSnapshot, PersonDossier};

pub(super) async fn cache_dossier_snapshot(
    pool: &PgPool,
    dossier: &PersonDossier,
) -> Result<DossierSnapshot, InvestigatorError> {
    let dossier_value = serde_json::to_value(dossier)?;
    let source_refs = serde_json::to_value(&dossier.source_refs)?;
    let snapshot_id = dossier_snapshot_id(&dossier.person.person_id);
    let row = sqlx::query(
        r#"
        INSERT INTO persona_dossier_snapshots (
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            generated_at
        )
        VALUES ($1, $2, $3, $4, 'suggested', $5)
        ON CONFLICT (persona_id)
        DO UPDATE SET
            dossier = EXCLUDED.dossier,
            source_refs = EXCLUDED.source_refs,
            generated_at = EXCLUDED.generated_at,
            updated_at = now()
        RETURNING
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            reviewed_by,
            reviewed_at,
            metadata,
            generated_at,
            created_at,
            updated_at
        "#,
    )
    .bind(&snapshot_id)
    .bind(&dossier.person.person_id)
    .bind(dossier_value)
    .bind(source_refs)
    .bind(dossier.generated_at)
    .fetch_one(pool)
    .await?;

    row_to_dossier_snapshot(row)
}

pub(super) async fn review_dossier_snapshot(
    pool: &PgPool,
    person_id: &str,
    review_state: DossierReviewState,
) -> Result<DossierSnapshot, InvestigatorError> {
    let row = sqlx::query(
        r#"
        UPDATE persona_dossier_snapshots
        SET
            review_state = $2,
            reviewed_by = 'owner_persona',
            reviewed_at = now(),
            updated_at = now()
        WHERE persona_id = $1
        RETURNING
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            reviewed_by,
            reviewed_at,
            metadata,
            generated_at,
            created_at,
            updated_at
        "#,
    )
    .bind(person_id)
    .bind(review_state.as_str())
    .fetch_optional(pool)
    .await?
    .ok_or(InvestigatorError::DossierSnapshotNotFound)?;

    row_to_dossier_snapshot(row)
}

fn dossier_snapshot_id(person_id: &str) -> String {
    format!("persona_dossier:v1:{person_id}")
}

fn row_to_dossier_snapshot(row: PgRow) -> Result<DossierSnapshot, InvestigatorError> {
    Ok(DossierSnapshot {
        dossier_snapshot_id: row.try_get("dossier_snapshot_id")?,
        persona_id: row.try_get("persona_id")?,
        dossier: row.try_get("dossier")?,
        source_refs: row.try_get("source_refs")?,
        review_state: DossierReviewState::parse(
            row.try_get::<String, _>("review_state")?.as_str(),
        )?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        metadata: row.try_get("metadata")?,
        generated_at: row.try_get("generated_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
