use super::super::errors::AiError;
use super::super::helpers::{halfvec_literal, validate_limit, validate_non_empty};
use super::models::SemanticSearchResult;
use super::rows::row_to_semantic_search_result;
use super::store::SemanticEmbeddingStore;

impl SemanticEmbeddingStore {
    pub async fn search(
        &self,
        embedding_model: &str,
        query_embedding: &[f32],
        limit: i64,
    ) -> Result<Vec<SemanticSearchResult>, AiError> {
        let embedding_model = validate_non_empty("embedding_model", embedding_model)?;
        let limit = validate_limit(limit)?;
        let vector_literal = halfvec_literal(query_embedding)?;

        let rows = sqlx::query(
            r#"
            SELECT
                source_kind,
                source_id,
                observation_id,
                title,
                source_text,
                graph_node_id,
                (1.0 - (embedding <=> $2::halfvec))::DOUBLE PRECISION AS score
            FROM semantic_embeddings
            WHERE embedding_model = $1
            ORDER BY embedding <=> $2::halfvec ASC, updated_at DESC, source_id
            LIMIT $3
            "#,
        )
        .bind(&embedding_model)
        .bind(vector_literal)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_semantic_search_result)
            .collect()
    }

    pub(in crate::ai::core) async fn text_search(
        &self,
        embedding_model: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<SemanticSearchResult>, AiError> {
        let embedding_model = validate_non_empty("embedding_model", embedding_model)?;
        let query = validate_non_empty("query", query)?;
        let limit = validate_limit(limit)?;

        let rows = sqlx::query(
            r#"
            WITH query AS (
                SELECT plainto_tsquery('simple', $2) AS ts_query
            )
            SELECT
                source_kind,
                source_id,
                observation_id,
                title,
                source_text,
                graph_node_id,
                ts_rank_cd(
                    to_tsvector('simple', title || ' ' || source_text),
                    query.ts_query
                )::DOUBLE PRECISION AS score
            FROM semantic_embeddings, query
            WHERE embedding_model = $1
              AND to_tsvector('simple', title || ' ' || source_text) @@ query.ts_query
            ORDER BY score DESC, updated_at DESC, source_id
            LIMIT $3
            "#,
        )
        .bind(&embedding_model)
        .bind(&query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_semantic_search_result)
            .collect()
    }
}
