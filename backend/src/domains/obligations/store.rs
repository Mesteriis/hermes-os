use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::ObligationStoreError;
use super::graph_projection::project_obligation_graph_in_transaction;
use super::ids::{evidence_id, obligation_id};
use super::models::{
    NewObligation, NewObligationEvidence, Obligation, ObligationEntityKind, ObligationReviewState,
};
use super::row_mapping::row_to_obligation;
use super::validation::{validate_non_empty, validate_obligation_with_evidence};

#[derive(Clone)]
pub struct ObligationStore {
    pool: PgPool,
}

impl ObligationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        validate_obligation_with_evidence(obligation, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, obligation, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        let obligation_id = obligation_id(obligation);
        let row = sqlx::query(
            r#"
            INSERT INTO obligations (
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                CAST($12 AS NUMERIC(5,4)),
                $13
            )
            ON CONFLICT (obligation_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                review_state = EXCLUDED.review_state,
                due_at = EXCLUDED.due_at,
                condition = EXCLUDED.condition,
                risk_state = EXCLUDED.risk_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&obligation_id)
        .bind(obligation.obligated_entity_kind.as_str())
        .bind(&obligation.obligated_entity_id)
        .bind(obligation.beneficiary_entity_kind.map(|kind| kind.as_str()))
        .bind(&obligation.beneficiary_entity_id)
        .bind(&obligation.statement)
        .bind(obligation.status.as_str())
        .bind(obligation.review_state.as_str())
        .bind(obligation.due_at)
        .bind(&obligation.condition)
        .bind(obligation.risk_state.as_str())
        .bind(obligation.confidence)
        .bind(&obligation.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_obligation(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&obligation_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO obligation_evidence (
                    evidence_id,
                    obligation_id,
                    source_kind,
                    source_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, CAST($6 AS NUMERIC(5,4)), $7)
                ON CONFLICT (obligation_id, source_kind, source_id)
                DO UPDATE SET
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&obligation_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        project_obligation_graph_in_transaction(transaction, &stored, evidence).await?;

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: ObligationEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE (obligated_entity_kind = $1 AND obligated_entity_id = $2)
               OR (beneficiary_entity_kind = $1 AND beneficiary_entity_id = $2)
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: ObligationReviewState,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE review_state = $1
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn set_review_state(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationStoreError> {
        validate_non_empty("obligation_id", obligation_id)?;
        let row = sqlx::query(
            r#"
            UPDATE obligations
            SET
                review_state = $1,
                updated_at = now()
            WHERE obligation_id = $2
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(obligation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ObligationStoreError::ObligationNotFound)?;

        row_to_obligation(row)
    }
}
