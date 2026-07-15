use serde_json::json;

use super::OllamaClient;
use super::error::OllamaError;
use super::models::OllamaEmbedResult;
use super::responses::EmbedResponse;

impl OllamaClient {
    pub async fn embed(&self, input: &str) -> Result<OllamaEmbedResult, OllamaError> {
        self.embed_with_model(input, &self.embed_model).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<OllamaEmbedResult, OllamaError> {
        if model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "input": input,
        });
        let response: EmbedResponse = self.post_json("/api/embed", &body).await?;
        let embedding = response
            .embeddings
            .and_then(|mut embeddings| {
                if embeddings.is_empty() {
                    None
                } else {
                    Some(embeddings.remove(0))
                }
            })
            .or(response.embedding)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama embed response omitted embeddings".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama embed response returned an empty vector".to_owned(),
            ));
        }

        Ok(OllamaEmbedResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            embedding,
            total_duration_ns: response.total_duration,
        })
    }
}
