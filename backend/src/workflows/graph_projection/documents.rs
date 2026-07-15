use serde_json::json;

use crate::domains::graph::core::models::NewGraphNode;
use crate::platform::graph::GraphNodeKind;

use super::errors::GraphProjectionError;
use super::models::{DocumentRow, GraphProjectionReport};
use super::rows::row_to_document;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_documents(&self) -> Result<Vec<DocumentRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, source_fingerprint, observation_id, imported_at
            FROM documents
            ORDER BY document_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_document).collect()
    }

    pub(super) async fn project_document(
        &self,
        document: &DocumentRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        self.graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Document,
                    &document.document_id,
                    &document.title,
                )
                .properties(json!({
                    "document_kind": document.document_kind,
                    "source_fingerprint": document.source_fingerprint,
                    "imported_at": document.imported_at,
                })),
            )
            .await?;
        report.nodes_upserted += 1;

        Ok(())
    }
}
