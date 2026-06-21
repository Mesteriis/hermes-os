use serde::Deserialize;
use serde_json::json;

use super::models::OmniRouteChatResult;
use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    pub async fn chat(&self, prompt: &str) -> Result<OmniRouteChatResult, OmniRouteError> {
        self.chat_with_model(prompt, &self.chat_model).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<OmniRouteChatResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "chat model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatCompletionsResponse = self.post_json("chat/completions", &body).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| {
                OmniRouteError::Protocol(
                    "OmniRoute chat response omitted assistant content".to_owned(),
                )
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute chat response content is empty".to_owned(),
            ));
        }

        Ok(OmniRouteChatResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            content,
        })
    }
}

fn strip_thinking_content(content: &str) -> String {
    let mut sanitized = content.trim().to_owned();
    while let Some(start) = sanitized.find("<think>") {
        let Some(end_offset) = sanitized[start..].find("</think>") else {
            sanitized.replace_range(start.., "");
            break;
        };
        let end = start + end_offset + "</think>".len();
        sanitized.replace_range(start..end, "");
    }

    if let Some(end) = sanitized.rfind("</think>") {
        sanitized = sanitized[end + "</think>".len()..].to_owned();
    }

    sanitized.trim().to_owned()
}

#[derive(Deserialize)]
struct ChatCompletionsResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}
