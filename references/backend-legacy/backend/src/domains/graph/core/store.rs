use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::GraphStoreError;
use super::ids::{edge_id, evidence_id};
use super::models::{GraphEdge, GraphNode, NewGraphEdge, NewGraphEvidence, NewGraphNode};
use super::row_mapping::{row_to_edge, row_to_node};
use super::validation::validate_edge_with_evidence;
use crate::platform::graph::node_id;

#[derive(Clone)]
pub struct GraphStore {
    pub(super) pool: PgPool,
}

impl GraphStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_node(&self, node: &NewGraphNode) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&self.pool)
        .await?;

        row_to_node(row)
    }

    pub async fn upsert_edge_with_evidence(
        &self,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;
        let mut transaction = self.pool.begin().await?;
        let stored_edge =
            Self::upsert_edge_with_evidence_in_transaction(&mut transaction, edge, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored_edge)
    }

    pub(crate) async fn upsert_node_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&mut **transaction)
        .await?;

        row_to_node(row)
    }

    pub(crate) async fn upsert_edge_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;

        let edge_id = edge_id(
            &edge.source_node_id,
            edge.relationship_type,
            &edge.target_node_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO graph_edges (
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence,
                review_state,
                properties,
                valid_from,
                valid_to
            )
            VALUES ($1, $2, $3, $4, CAST($5 AS NUMERIC(5,4)), $6, $7, $8, $9)
            ON CONFLICT (source_node_id, target_node_id, relationship_type) WHERE valid_to IS NULL
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                properties = EXCLUDED.properties,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                updated_at = now()
            RETURNING
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            "#,
        )
        .bind(&edge_id)
        .bind(&edge.source_node_id)
        .bind(&edge.target_node_id)
        .bind(edge.relationship_type.as_str())
        .bind(edge.confidence)
        .bind(edge.review_state.as_str())
        .bind(&edge.properties)
        .bind(edge.valid_from)
        .bind(edge.valid_to)
        .fetch_one(&mut **transaction)
        .await?;

        for item in evidence {
            let evidence_id = evidence_id(&edge_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO graph_evidence (
                    evidence_id,
                    edge_id,
                    source_kind,
                    source_id,
                    observation_id,
                    excerpt,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (edge_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = COALESCE(EXCLUDED.observation_id, graph_evidence.observation_id),
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&edge_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        row_to_edge(row)
    }
}
