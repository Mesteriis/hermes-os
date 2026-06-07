use crate::integrations::ollama::client::{OllamaClient, OllamaError};
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
    ollama: Option<OllamaClient>,
}

impl MultilingualService {
    pub fn new(ollama: Option<OllamaClient>) -> Self {
        Self { ollama }
    }

    /// Heuristic language detection based on character sets and common words.
    pub fn detect_language(text: &str) -> LanguageDetection {
        let text = text.trim();
        if text.is_empty() {
            return LanguageDetection {
                language: "unknown".into(),
                confidence: 0.0,
                script: None,
            };
        }

        let has_cyrillic = text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
        let has_spanish = text.to_lowercase().contains('ñ');
        let has_latin = text.chars().any(|c| c.is_ascii_alphabetic());
        let has_cjk = text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));

        let lower = text.to_lowercase();

        if has_cyrillic {
            // Distinguish Russian vs Ukrainian
            if lower.contains('ї') || lower.contains('є') {
                return LanguageDetection {
                    language: "uk".into(),
                    confidence: 0.85,
                    script: Some("cyrillic".into()),
                };
            }
            return LanguageDetection {
                language: "ru".into(),
                confidence: 0.90,
                script: Some("cyrillic".into()),
            };
        }
        if has_spanish {
            return LanguageDetection {
                language: "es".into(),
                confidence: 0.85,
                script: Some("latin".into()),
            };
        }
        if has_cjk {
            return LanguageDetection {
                language: "zh".into(),
                confidence: 0.70,
                script: Some("cjk".into()),
            };
        }

        // Check common words
        let spanish_words = [
            "hola",
            "gracias",
            "para",
            "como",
            "que",
            "por favor",
            "saludos",
            "adjunto",
        ];
        let russian_latin = ["privet", "spasibo", "pozhaluysta"];
        let german_words = [
            "mit", "und", "der", "die", "das", "ist", "von", "für", "danke", "bitte",
        ];

        let spanish_score = spanish_words.iter().filter(|w| lower.contains(*w)).count() as f32
            / spanish_words.len() as f32;
        let russian_latin_score = russian_latin.iter().filter(|w| lower.contains(*w)).count()
            as f32
            / russian_latin.len() as f32;
        let german_score = german_words.iter().filter(|w| lower.contains(*w)).count() as f32
            / german_words.len() as f32;

        if spanish_score > 0.1 {
            return LanguageDetection {
                language: "es".into(),
                confidence: 0.70,
                script: Some("latin".into()),
            };
        }
        if russian_latin_score > 0.1 {
            return LanguageDetection {
                language: "ru".into(),
                confidence: 0.55,
                script: Some("latin".into()),
            };
        }
        if german_score > 0.1 {
            return LanguageDetection {
                language: "de".into(),
                confidence: 0.65,
                script: Some("latin".into()),
            };
        }
        if has_latin {
            return LanguageDetection {
                language: "en".into(),
                confidence: 0.50,
                script: Some("latin".into()),
            };
        }

        LanguageDetection {
            language: "unknown".into(),
            confidence: 0.0,
            script: None,
        }
    }

    /// Translate text using LLM. Falls back to identity if no LLM.
    pub async fn translate(
        &self,
        text: &str,
        target_lang: &str,
    ) -> Result<Option<Translation>, MultilingualError> {
        let Some(ref ollama) = self.ollama else {
            return Ok(None);
        };
        let prompt = format!(
            "Translate the following text to {target_lang}. Return ONLY the translated text, no explanations:\n\n{text}"
        );
        let result = ollama.chat(&prompt).await?;
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
    Ollama(#[from] OllamaError),
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
