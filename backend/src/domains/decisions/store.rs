use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::workflows::review_mirror::sync_decision_review_state_in_transaction;

use super::errors::DecisionStoreError;
use super::evidence::{
    link_decision_review_transition_in_transaction, link_decision_support_in_transaction,
};
use super::graph_projection::project_decision_graph_in_transaction;
use super::ids::{decision_id, evidence_id};
use super::models::{
    Decision, DecisionEntityKind, DecisionReviewState, NewDecision, NewDecisionEvidence,
    NewDecisionImpactedEntity,
};
use super::row_mapping::row_to_decision;
use super::validation::{validate_decision_with_evidence, validate_non_empty};

#[derive(Clone)]
pub struct DecisionStore {
    pub(super) pool: PgPool,
}

impl DecisionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        validate_decision_with_evidence(decision, evidence, impacted_entities)?;

        let mut transaction = self.pool.begin().await?;
        let stored = Self::upsert_with_evidence_in_transaction(
            &mut transaction,
            decision,
            evidence,
            impacted_entities,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let decision_id = decision_id(decision);
        let row = sqlx::query(
            r#"
            INSERT INTO decisions (
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
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
                CAST($10 AS NUMERIC(5,4)),
                $11
            )
            ON CONFLICT (decision_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                rationale = EXCLUDED.rationale,
                alternatives = EXCLUDED.alternatives,
                decided_by_entity_kind = EXCLUDED.decided_by_entity_kind,
                decided_by_entity_id = EXCLUDED.decided_by_entity_id,
                decided_at = EXCLUDED.decided_at,
                review_state = EXCLUDED.review_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&decision_id)
        .bind(&decision.title)
        .bind(decision.status.as_str())
        .bind(&decision.rationale)
        .bind(&decision.alternatives)
        .bind(decision.decided_by_entity_kind.map(|kind| kind.as_str()))
        .bind(&decision.decided_by_entity_id)
        .bind(decision.decided_at)
        .bind(decision.review_state.as_str())
        .bind(decision.confidence)
        .bind(&decision.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_decision(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&decision_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO decision_evidence (
                    evidence_id,
                    decision_id,
                    source_kind,
                    source_id,
                    observation_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS NUMERIC(5,4)), $8)
                ON CONFLICT (decision_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&decision_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_decision_support_in_transaction(
                    transaction,
                    observation_id,
                    decision_id.clone(),
                    item.confidence,
                    json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    }),
                )
                .await?;
            }
        }

        for item in impacted_entities {
            sqlx::query(
                r#"
                INSERT INTO decision_impacted_entities (
                    decision_id,
                    entity_kind,
                    entity_id,
                    impact_type,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (decision_id, entity_kind, entity_id)
                DO UPDATE SET
                    impact_type = EXCLUDED.impact_type,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&decision_id)
            .bind(item.entity_kind.as_str())
            .bind(&item.entity_id)
            .bind(&item.impact_type)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        project_decision_graph_in_transaction(transaction, &stored, evidence, impacted_entities)
            .await?;

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: DecisionEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                decision.decision_id,
                decision.title,
                decision.status,
                decision.rationale,
                decision.alternatives,
                decision.decided_by_entity_kind,
                decision.decided_by_entity_id,
                decision.decided_at,
                decision.review_state,
                decision.confidence::float8 AS confidence,
                decision.metadata,
                decision.created_at,
                decision.updated_at
            FROM decisions decision
            LEFT JOIN decision_impacted_entities impacted
              ON impacted.decision_id = decision.decision_id
            WHERE (decision.decided_by_entity_kind = $1 AND decision.decided_by_entity_id = $2)
               OR (impacted.entity_kind = $1 AND impacted.entity_id = $2)
            ORDER BY decision.updated_at DESC, decision.decision_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: DecisionReviewState,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM decisions
            WHERE review_state = $1
            ORDER BY updated_at DESC, decision_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn set_review_state(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionStoreError> {
        self.set_review_state_with_observation(decision_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Decision, DecisionStoreError> {
        validate_non_empty("decision_id", decision_id)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE decisions
            SET
                review_state = $1,
                updated_at = now()
            WHERE decision_id = $2
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(decision_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(DecisionStoreError::DecisionNotFound)?;

        let decision = row_to_decision(row)?;
        link_decision_review_transition_in_transaction(
            &mut transaction,
            observation_id,
            &decision.decision_id,
            decision.review_state,
            metadata,
        )
        .await?;
        sync_decision_review_state_in_transaction(&mut transaction, &decision).await?;
        transaction.commit().await?;
        Ok(decision)
    }
}

async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewDecisionEvidence],
) -> Result<(), DecisionStoreError> {
    let observation_ids: Vec<String> = evidence
        .iter()
        .filter_map(|item| item.observation_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if observation_ids.is_empty() {
        return Ok(());
    }

    let stored_observation_ids: HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observations
        WHERE observation_id = ANY($1)
        "#,
    )
    .bind(&observation_ids)
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .collect();

    for observation_id in observation_ids {
        if !stored_observation_ids.contains(&observation_id) {
            return Err(DecisionStoreError::ObservationNotFound(observation_id));
        }
    }

    Ok(())
}
