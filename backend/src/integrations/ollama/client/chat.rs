use serde_json::json;

use super::OllamaClient;
use super::error::OllamaError;
use super::models::OllamaChatResult;
use super::responses::ChatResponse;
use super::sanitization::strip_thinking_content;

impl OllamaClient {
    pub async fn chat(&self, prompt: &str) -> Result<OllamaChatResult, OllamaError> {
        self.chat_with_model(prompt, &self.chat_model).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<OllamaChatResult, OllamaError> {
        if model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("chat model is empty".to_owned()));
        }
        let body = json!({
            "model": model,
            "stream": false,
            "think": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatResponse = self.post_json("/api/chat", &body).await?;
        let content = response
            .message
            .and_then(|message| message.content)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama chat response omitted assistant content".to_owned())
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama chat response content is empty".to_owned(),
            ));
        }

        Ok(OllamaChatResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            content,
            total_duration_ns: response.total_duration,
        })
    }
}
