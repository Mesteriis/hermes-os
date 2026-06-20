use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationFingerprint {
    pub avg_message_length: Option<usize>,
    pub avg_response_hours: Option<f64>,
    pub frequent_topics: Vec<String>,
    pub typical_tone: Option<String>,
    pub detected_language: Option<String>,
    pub writing_style: Option<String>,
    pub preferred_time_of_day: Option<String>,
    pub trust_score: Option<i16>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonInsight {
    pub person_id: String,
    pub fingerprint: CommunicationFingerprint,
    pub suggested_actions: Vec<String>,
}

#[derive(Clone)]
pub struct PersonIntelligenceService {
    runtime: Option<AiRuntimeClient>,
}

impl PersonIntelligenceService {
    pub fn new(runtime: Option<AiRuntimeClient>) -> Self {
        Self { runtime }
    }

    pub fn heuristic_fingerprint(messages: &[PersonMessage]) -> CommunicationFingerprint {
        if messages.is_empty() {
            return CommunicationFingerprint {
                avg_message_length: None,
                avg_response_hours: None,
                frequent_topics: vec![],
                typical_tone: None,
                detected_language: None,
                writing_style: None,
                preferred_time_of_day: None,
                trust_score: None,
            };
        }

        let total_len: usize = messages.iter().map(|m| m.body_text.len()).sum();
        let avg_len = total_len / messages.len();

        let mut topics = Vec::new();
        let combined_text: String = messages
            .iter()
            .map(|m| &m.body_text as &str)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        for (topic, keywords) in [
            ("finance", &["invoice", "payment", "amount", "tax"][..]),
            ("legal", &["contract", "nda", "agreement", "legal"][..]),
            (
                "project",
                &["project", "deadline", "milestone", "deliverable"][..],
            ),
            ("support", &["help", "issue", "problem", "bug"][..]),
        ] {
            if keywords.iter().any(|k| combined_text.contains(k)) {
                topics.push(topic.into());
            }
        }

        let tone = if combined_text.contains("urgent") || combined_text.contains("asap") {
            Some("urgent".into())
        } else if combined_text.contains("thanks") || combined_text.contains("appreciate") {
            Some("friendly".into())
        } else if combined_text.contains("please") && combined_text.contains("would") {
            Some("polite".into())
        } else {
            Some("neutral".into())
        };

        let detected_language = detect_language(&combined_text);

        let trust = 50i16
            .saturating_add((messages.len() as i16 * 2).min(30))
            .saturating_add(if !topics.is_empty() { 10 } else { 0 });

        CommunicationFingerprint {
            avg_message_length: Some(avg_len),
            avg_response_hours: None,
            frequent_topics: topics,
            typical_tone: tone,
            detected_language: Some(detected_language),
            writing_style: if avg_len > 500 {
                Some("verbose".into())
            } else if avg_len < 100 {
                Some("concise".into())
            } else {
                Some("balanced".into())
            },
            preferred_time_of_day: None,
            trust_score: Some(trust.clamp(0, 100)),
        }
    }

    pub async fn llm_fingerprint(
        &self,
        messages: &[PersonMessage],
    ) -> Result<Option<CommunicationFingerprint>, PersonIntelligenceError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };
        let sample: String = messages
            .iter()
            .take(5)
            .map(|m| format!("Subject: {}\nBody: {}\n", m.subject, m.body_text))
            .collect::<Vec<_>>()
            .join("\n---\n");
        let prompt = format!(
            "Analyze communication patterns from these email samples. Return JSON with: frequent_topics (array of strings), typical_tone (one word), detected_language (code), writing_style (verbose/concise/balanced), preferred_time_of_day (morning/afternoon/evening or null).\n\nSamples:\n{sample}"
        );
        let result = runtime.chat(&prompt).await?;
        let content = result
            .content
            .trim()
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .map(str::trim)
            .unwrap_or(result.content.trim());
        Ok(serde_json::from_str(content).ok())
    }

    pub fn suggested_actions(fingerprint: &CommunicationFingerprint) -> Vec<String> {
        let mut actions = Vec::new();
        if let Some(ref tone) = fingerprint.typical_tone {
            actions.push(format!("Person tends to be {tone} — match tone in replies"));
        }
        if let Some(ref lang) = fingerprint.detected_language
            && lang != "en"
        {
            actions.push(format!(
                "Person writes in {lang} — consider translating replies"
            ));
        }
        if let Some(ref style) = fingerprint.writing_style {
            actions.push(format!("Person style: {style}"));
        }
        if let Some(score) = fingerprint.trust_score
            && score < 30
        {
            actions.push("Low trust score — verify claims".into());
        }
        actions
    }
}

fn detect_language(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return "unknown".to_owned();
    }

    let lower = text.to_lowercase();
    if text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
        if lower.contains('ї') || lower.contains('є') {
            return "uk".to_owned();
        }
        return "ru".to_owned();
    }
    if text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        return "zh".to_owned();
    }
    if lower.contains('ñ')
        || [
            "hola",
            "gracias",
            "para",
            "como",
            "que",
            "por favor",
            "saludos",
            "adjunto",
        ]
        .iter()
        .any(|word| lower.contains(word))
    {
        return "es".to_owned();
    }
    if ["privet", "spasibo", "pozhaluysta"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return "ru".to_owned();
    }
    if [
        "mit", "und", "der", "die", "das", "ist", "von", "für", "danke", "bitte",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        return "de".to_owned();
    }
    if text.chars().any(|c| c.is_ascii_alphabetic()) {
        return "en".to_owned();
    }

    "unknown".to_owned()
}

#[derive(Clone, Debug)]
pub struct PersonMessage {
    pub subject: String,
    pub body_text: String,
    pub occurred_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Error)]
pub enum PersonIntelligenceError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimeError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_messages() -> Vec<PersonMessage> {
        vec![
            PersonMessage {
                subject: "Invoice".into(),
                body_text: "Please pay invoice #123 for $500".into(),
                occurred_at: None,
            },
            PersonMessage {
                subject: "Thanks".into(),
                body_text: "Thank you for your help with the project".into(),
                occurred_at: None,
            },
        ]
    }

    #[test]
    fn fingerprint_detects_topics() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.frequent_topics.contains(&"finance".into()));
    }
    #[test]
    fn fingerprint_sets_trust() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.trust_score.unwrap() >= 50);
    }
    #[test]
    fn fingerprint_detects_tone() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.typical_tone.is_some());
    }
    #[test]
    fn empty_messages_returns_none() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&[]);
        assert!(fp.trust_score.is_none());
    }
}
