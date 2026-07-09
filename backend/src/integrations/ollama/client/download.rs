use serde_json::json;

use super::OllamaClient;
use super::error::OllamaError;
use super::responses::PullResponse;

impl OllamaClient {
    pub async fn pull_model(&self, model: &str) -> Result<(), OllamaError> {
        let response: PullResponse = self
            .post_json(
                "/api/pull",
                &json!({
                    "name": model,
                    "stream": false,
                }),
            )
            .await?;

        if let Some(error) = response.error.filter(|value| !value.trim().is_empty()) {
            return Err(OllamaError::Protocol(error));
        }

        let status = response.status.unwrap_or_default();
        if status.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama pull response omitted status".to_owned(),
            ));
        }

        Ok(())
    }
}
