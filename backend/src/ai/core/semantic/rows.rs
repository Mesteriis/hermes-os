use sqlx::Row;
use sqlx::postgres::PgRow;

use super::super::errors::AiError;
use super::models::{SemanticEmbedding, SemanticSearchResult, SemanticSourceKind};

pub(super) fn row_to_semantic_embedding(row: PgRow) -> Result<SemanticEmbedding, AiError> {
    let source_kind: String = row.try_get("source_kind")?;
    SemanticSourceKind::parse(&source_kind)?;
    Ok(SemanticEmbedding {
        semantic_embedding_id: row.try_get("semantic_embedding_id")?,
        source_kind,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        source_text: row.try_get("source_text")?,
        content_hash: row.try_get("content_hash")?,
        embedding_model: row.try_get("embedding_model")?,
        embedding_dimension: row.try_get("embedding_dimension")?,
        graph_node_id: row.try_get("graph_node_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_semantic_search_result(row: PgRow) -> Result<SemanticSearchResult, AiError> {
    let source_kind: String = row.try_get("source_kind")?;
    SemanticSourceKind::parse(&source_kind)?;
    Ok(SemanticSearchResult {
        source_kind,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        source_text: row.try_get("source_text")?,
        graph_node_id: row.try_get("graph_node_id")?,
        score: row.try_get("score")?,
    })
}
