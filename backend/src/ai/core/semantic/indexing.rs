use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::super::errors::AiError;
use super::super::helpers::content_hash;
use super::models::{NewSemanticEmbedding, SemanticIndexReport};
use super::store::SemanticEmbeddingStore;
use crate::integrations::ai_runtime::AiRuntimeClient;

impl SemanticEmbeddingStore {
    pub async fn index_canonical_sources(
        &self,
        runtime: &AiRuntimeClient,
        embedding_model: &str,
    ) -> Result<SemanticIndexReport, AiError> {
        let sources = self.canonical_sources().await?;
        let mut report = SemanticIndexReport::default();

        for source in sources {
            report.sources_seen += 1;
            let source_hash = content_hash(&source.source_text);
            if self
                .is_current(
                    source.source_kind,
                    &source.source_id,
                    embedding_model,
                    &source_hash,
                )
                .await?
            {
                report.sources_skipped += 1;
                continue;
            }

            let embedding = runtime
                .embed_with_model(&source.source_text, embedding_model)
                .await?;
            if embedding.embedding.len() != AI_EMBEDDING_DIMENSION {
                return Err(AiError::InvalidEmbeddingDimension {
                    expected: AI_EMBEDDING_DIMENSION,
                    actual: embedding.embedding.len(),
                });
            }
            self.upsert_embedding(NewSemanticEmbedding {
                source_kind: source.source_kind,
                source_id: &source.source_id,
                observation_id: source.observation_id.as_deref(),
                title: &source.title,
                source_text: &source.source_text,
                embedding_model,
                embedding: &embedding.embedding,
                graph_node_id: source.graph_node_id.as_deref(),
            })
            .await?;
            report.sources_indexed += 1;
        }

        Ok(report)
    }
}
