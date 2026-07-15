use super::super::errors::AiError;
use super::models::SemanticSource;
use super::source_documents::append_document_sources;
use super::source_messages::append_message_sources;
use super::source_personas::append_persona_sources;
use super::source_projects::append_project_sources;
use super::source_tasks::append_task_sources;
use super::store::SemanticEmbeddingStore;

impl SemanticEmbeddingStore {
    pub(super) async fn canonical_sources(&self) -> Result<Vec<SemanticSource>, AiError> {
        let mut sources = Vec::new();

        append_message_sources(&self.pool, &mut sources).await?;
        append_document_sources(&self.pool, &mut sources).await?;
        append_project_sources(&self.pool, &mut sources).await?;
        append_task_sources(&self.pool, &mut sources).await?;
        append_persona_sources(&self.pool, &mut sources).await?;

        Ok(sources)
    }
}
