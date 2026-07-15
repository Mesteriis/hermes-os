use super::OllamaClient;
use super::error::OllamaError;
use super::responses::{TagsResponse, VersionResponse};

impl OllamaClient {
    pub async fn version(&self) -> Result<String, OllamaError> {
        let response: VersionResponse = self.get_json("/api/version").await?;
        if response.version.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama version response omitted version".to_owned(),
            ));
        }
        Ok(response.version)
    }

    pub async fn tags(&self) -> Result<Vec<String>, OllamaError> {
        let response: TagsResponse = self.get_json("/api/tags").await?;
        Ok(response
            .models
            .into_iter()
            .map(|model| model.name)
            .filter(|name| !name.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OllamaError> {
        let tags = self.tags().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !tags.iter().any(|tag| tag == model) {
                return Err(OllamaError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }
}
