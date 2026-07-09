use crate::ai::hub::{AiHub, AiHubError, SharedAiHub};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanguageDetection {
    pub language: String,
    pub confidence: f32,
    pub script: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Translation {
    pub original_language: String,
    pub target_language: String,
    pub translated_text: String,
    pub model: String,
}

#[derive(Clone)]
pub struct MultilingualService {
    hub: Option<SharedAiHub>,
}

impl MultilingualService {
    pub fn new(hub: Option<SharedAiHub>) -> Self {
        Self { hub }
    }

    /// Heuristic language detection based on local Rust guards.
    pub fn detect_language(text: &str) -> LanguageDetection {
        let detection = AiHub::detect_language(text);
        LanguageDetection {
            language: detection.language,
            confidence: detection.confidence,
            script: detection.script,
        }
    }

    /// Translate text using LLM. Falls back to identity if no LLM.
    pub async fn translate(
        &self,
        text: &str,
        target_lang: &str,
    ) -> Result<Option<Translation>, MultilingualError> {
        let Some(ref hub) = self.hub else {
            return Ok(None);
        };
        let result = hub.translate_text(text, target_lang).await?;
        Ok(Some(Translation {
            original_language: "detected".into(),
            target_language: target_lang.into(),
            translated_text: result.content,
            model: result.model,
        }))
    }
}

#[derive(Debug, Error)]
pub enum MultilingualError {
    #[error(transparent)]
    Hub(#[from] AiHubError),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_russian() {
        let d = MultilingualService::detect_language("Привет, как дела?");
        assert_eq!(d.language, "ru");
    }
    #[test]
    fn detect_spanish() {
        let d = MultilingualService::detect_language("Hola, ¿cómo estás?");
        assert_eq!(d.language, "es");
    }
    #[test]
    fn detect_english() {
        let d = MultilingualService::detect_language("Hello, how are you?");
        assert_eq!(d.language, "en");
    }
    #[test]
    fn detect_spanish_words() {
        let d = MultilingualService::detect_language("Gracias por su ayuda, saludos cordiales");
        assert_eq!(d.language, "es");
    }
    #[test]
    fn detect_empty() {
        let d = MultilingualService::detect_language("");
        assert_eq!(d.language, "unknown");
    }
}
