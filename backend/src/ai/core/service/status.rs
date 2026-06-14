use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::super::types::AiStatusResponse;
use super::core::AiService;

impl AiService {
    pub async fn status(&self) -> AiStatusResponse {
        let version = self.runtime.version().await;
        let models = self.runtime.models().await;
        let chat_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.default_chat)
            })
            .unwrap_or(false);
        let embedding_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.embeddings)
            })
            .unwrap_or(false);

        AiStatusResponse {
            runtime: self.runtime.runtime_name().to_owned(),
            status: if version.is_ok()
                && models.is_ok()
                && chat_model_available
                && embedding_model_available
            {
                "ok"
            } else {
                "unavailable"
            }
            .to_owned(),
            version: version.ok().flatten(),
            chat_model: self.model_routing.default_chat.clone(),
            embedding_model: self.model_routing.embeddings.clone(),
            embedding_dimension: AI_EMBEDDING_DIMENSION,
            chat_model_available,
            embedding_model_available,
        }
    }
}
