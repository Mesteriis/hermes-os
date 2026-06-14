use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::domains::graph::core::{GraphNodeKind, node_id};

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_document_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT document_id, title, extracted_text
        FROM documents
        WHERE length(trim(extracted_text)) > 0
        ORDER BY imported_at DESC, document_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let document_id: String = row.try_get("document_id")?;
        let title: String = row.try_get("title")?;
        let extracted_text: String = row.try_get("extracted_text")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Document,
            source_id: document_id.clone(),
            title: title.clone(),
            source_text: format!("{title}\n\n{extracted_text}"),
            graph_node_id: Some(node_id(GraphNodeKind::Document, &document_id)),
        });
    }

    Ok(())
}
