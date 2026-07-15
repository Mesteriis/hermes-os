use super::super::constants::{AI_EMBEDDING_DIMENSION, DEFAULT_RETRIEVAL_LIMIT};
use super::super::errors::AiError;
use super::super::helpers::merge_retrieval_results;
use super::super::semantic::store::SemanticEmbeddingStore;
use super::super::types::AiCitation;
use super::core::AiService;

impl AiService {
    pub(super) async fn retrieve_citations(&self, query: &str) -> Result<Vec<AiCitation>, AiError> {
        let semantic_store = SemanticEmbeddingStore::new(self.pool.clone());
        let embedding_model = &self.model_routing.embeddings;
        semantic_store
            .index_canonical_sources(&self.hub, embedding_model)
            .await?;
        let query_embedding = self.hub.embed(query).await?;
        if query_embedding.embedding.len() != AI_EMBEDDING_DIMENSION {
            return Err(AiError::InvalidEmbeddingDimension {
                expected: AI_EMBEDDING_DIMENSION,
                actual: query_embedding.embedding.len(),
            });
        }

        let vector_results = semantic_store
            .search(
                embedding_model,
                &query_embedding.embedding,
                DEFAULT_RETRIEVAL_LIMIT,
            )
            .await?;
        let text_results = semantic_store
            .text_search(embedding_model, query, DEFAULT_RETRIEVAL_LIMIT)
            .await?;
        let merged = merge_retrieval_results(vector_results, text_results);

        Ok(merged
            .into_iter()
            .take(DEFAULT_RETRIEVAL_LIMIT as usize)
            .map(AiCitation::from)
            .collect())
    }
}
