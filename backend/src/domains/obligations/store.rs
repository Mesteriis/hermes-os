use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};

use super::errors::ObligationStoreError;
use super::evidence::{
    link_obligation_review_transition_in_transaction, link_obligation_support_in_transaction,
};
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
        validate_evidence_observations_exist(transaction, evidence).await?;
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
                    observation_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS NUMERIC(5,4)), $8)
                ON CONFLICT (obligation_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&obligation_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_obligation_support_in_transaction(
                    transaction,
                    observation_id,
                    obligation_id.clone(),
                    item.confidence,
                    json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    }),
                )
                .await?;
            }
        }

        upsert_obligation_graph_projection_in_transaction(transaction, &stored, evidence).await?;

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
        self.set_review_state_with_observation(obligation_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Obligation, ObligationStoreError> {
        validate_non_empty("obligation_id", obligation_id)?;
        let mut transaction = self.pool.begin().await?;
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
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(ObligationStoreError::ObligationNotFound)?;

        let obligation = row_to_obligation(row)?;
        link_obligation_review_transition_in_transaction(
            &mut transaction,
            observation_id,
            &obligation.obligation_id,
            obligation.review_state,
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(obligation)
    }
}

async fn upsert_obligation_graph_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    let obligation_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            GraphNodeKind::Obligation,
            obligation.obligation_id.clone(),
            obligation.statement.clone(),
        )
        .properties(json!({
            "domain": "obligation",
            "obligation_id": obligation.obligation_id,
            "status": obligation.status.as_str(),
            "review_state": obligation.review_state.as_str(),
        })),
    )
    .await?;

    let graph_review_state = obligation_review_state(obligation.review_state);
    let graph_evidence = graph_obligation_evidence(obligation, evidence);

    upsert_obligation_entity_graph_edge_in_transaction(
        transaction,
        &obligation_node.node_id,
        obligation.obligated_entity_kind,
        &obligation.obligated_entity_id,
        "obligated_entity",
        obligation.confidence,
        graph_review_state,
        &graph_evidence,
    )
    .await?;

    if let (Some(beneficiary_entity_kind), Some(beneficiary_entity_id)) = (
        obligation.beneficiary_entity_kind,
        obligation.beneficiary_entity_id.as_deref(),
    ) {
        upsert_obligation_entity_graph_edge_in_transaction(
            transaction,
            &obligation_node.node_id,
            beneficiary_entity_kind,
            beneficiary_entity_id,
            "beneficiary_entity",
            obligation.confidence,
            graph_review_state,
            &graph_evidence,
        )
        .await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn upsert_obligation_entity_graph_edge_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation_node_id: &str,
    entity_kind: ObligationEntityKind,
    entity_id: &str,
    link_role: &str,
    confidence: f64,
    review_state: GraphReviewState,
    graph_evidence: &NewGraphEvidence,
) -> Result<(), ObligationStoreError> {
    let target_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            obligation_entity_graph_node_kind(entity_kind),
            entity_id,
            entity_id,
        )
        .properties(json!({
            "domain": "obligation",
            "entity_kind": entity_kind.as_str(),
            "entity_id": entity_id,
        })),
    )
    .await?;
    let edge = NewGraphEdge::new(
        obligation_node_id.to_owned(),
        target_node.node_id,
        RelationshipType::EntityRelationship,
        confidence,
        review_state,
    )
    .properties(json!({
        "domain": "obligation",
        "link_role": link_role,
    }));
    let _ = GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
        transaction,
        &edge,
        std::slice::from_ref(graph_evidence),
    )
    .await?;
    Ok(())
}

fn graph_obligation_evidence(
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Obligation,
        obligation.obligation_id.clone(),
    )
    .metadata(json!({ "domain": "obligation" }));

    if let Some(item) = evidence.first() {
        graph_evidence = graph_evidence.metadata(json!({
            "domain": "obligation",
            "source_kind": item.source_kind.as_str(),
            "source_id": item.source_id,
        }));
        if let Some(quote) = &item.quote {
            graph_evidence = graph_evidence.excerpt(quote.clone());
        }
        if let Some(observation_id) = &item.observation_id {
            graph_evidence = graph_evidence.observation_id(observation_id.clone());
        }
    }

    graph_evidence
}

fn obligation_review_state(review_state: ObligationReviewState) -> GraphReviewState {
    match review_state {
        ObligationReviewState::Suggested => GraphReviewState::Suggested,
        ObligationReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ObligationReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn obligation_entity_graph_node_kind(entity_kind: ObligationEntityKind) -> GraphNodeKind {
    match entity_kind {
        ObligationEntityKind::Persona => GraphNodeKind::Person,
        ObligationEntityKind::Organization => GraphNodeKind::Organization,
        ObligationEntityKind::Project => GraphNodeKind::Project,
        ObligationEntityKind::Communication => GraphNodeKind::Message,
        ObligationEntityKind::Document => GraphNodeKind::Document,
        ObligationEntityKind::Task => GraphNodeKind::Task,
        ObligationEntityKind::Event => GraphNodeKind::Event,
        ObligationEntityKind::Decision => GraphNodeKind::Decision,
        ObligationEntityKind::Obligation => GraphNodeKind::Obligation,
        ObligationEntityKind::Knowledge => GraphNodeKind::Knowledge,
    }
}

async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
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
            return Err(ObligationStoreError::ObservationNotFound(observation_id));
        }
    }

    Ok(())
}
