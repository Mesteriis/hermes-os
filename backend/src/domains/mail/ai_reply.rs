use crate::domains::mail::messages::ProjectedMessage;
use crate::integrations::ollama::client::{OllamaClient, OllamaError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiReplyDraft {
    pub subject: String,
    pub body: String,
    pub tone: String,
    pub language: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiReplyOptions {
    pub tone: Option<String>,
    pub language: Option<String>,
    pub context: Option<String>,
}

#[derive(Clone)]
pub struct AiReplyService {
    ollama: Option<OllamaClient>,
}

impl AiReplyService {
    pub fn new(ollama: Option<OllamaClient>) -> Self {
        Self { ollama }
    }

    pub async fn generate_reply(
        &self,
        message: &ProjectedMessage,
        options: &AiReplyOptions,
    ) -> Result<Option<AiReplyDraft>, AiReplyError> {
        let Some(ref ollama) = self.ollama else {
            return Ok(None);
        };
        let tone = options.tone.as_deref().unwrap_or("professional");
        let lang = options.language.as_deref().unwrap_or("auto-detect");
        let context = options.context.as_deref().unwrap_or("");

        let prompt = format!(
            "You are replying to an email.\n\nOriginal email:\nFrom: {}\nSubject: {}\nBody:\n{}\n\n{}\nGenerate a reply in {lang} with a {tone} tone. Return ONLY the reply body text, no subject line, no explanations.",
            message.sender,
            message.subject,
            truncate(&message.body_text, 2000),
            if context.is_empty() {
                "".into()
            } else {
                format!("Additional context: {context}")
            },
        );

        let result = ollama.chat(&prompt).await?;
        let body = result.content.trim().to_owned();

        let subject = if message.subject.to_lowercase().starts_with("re:") {
            message.subject.clone()
        } else {
            format!("Re: {}", message.subject)
        };

        Ok(Some(AiReplyDraft {
            subject,
            body,
            tone: tone.into(),
            language: lang.into(),
        }))
    }

    pub async fn generate_reply_variants(
        &self,
        message: &ProjectedMessage,
        languages: &[String],
        tones: &[String],
    ) -> Result<Vec<AiReplyDraft>, AiReplyError> {
        let mut variants = Vec::new();
        for lang in languages {
            for tone in tones {
                if let Some(draft) = self
                    .generate_reply(
                        message,
                        &AiReplyOptions {
                            language: Some(lang.clone()),
                            tone: Some(tone.clone()),
                            context: None,
                        },
                    )
                    .await?
                {
                    variants.push(draft);
                }
            }
        }
        Ok(variants)
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

#[derive(Debug, Error)]
pub enum AiReplyError {
    #[error(transparent)]
    Ollama(#[from] OllamaError),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn truncate_short() {
        assert_eq!(truncate("hi", 10), "hi");
    }
    #[test]
    fn truncate_long() {
        assert_eq!(truncate("hello world long text", 5), "hello");
    }
}
