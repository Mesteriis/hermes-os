use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailAnalysis {
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub deadlines: Vec<String>,
    #[serde(default)]
    pub event_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub persona_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub organization_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub document_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub agreement_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub task_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub decision_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub obligation_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub relationship_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub fact_candidates: Vec<EmailKnowledgeCandidate>,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub suggested_action: Option<String>,
    pub extracted_deadline: Option<String>,
    pub language: Option<String>,
    pub model: String,
    pub prompt_version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmailKnowledgeCandidate {
    pub title: String,
    pub evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub identifiers: Value,
}

impl EmailKnowledgeCandidate {
    pub fn new(title: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            evidence: evidence.into(),
            kind: None,
            summary: None,
            confidence: None,
            source_message_id: None,
            identifiers: Value::Null,
        }
    }

    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn source_message_id(mut self, source_message_id: impl Into<String>) -> Self {
        self.source_message_id = Some(source_message_id.into());
        self
    }

    pub fn identifiers(mut self, identifiers: Value) -> Self {
        self.identifiers = identifiers;
        self
    }

    pub fn summary_text(&self) -> String {
        self.summary
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.evidence)
            .trim()
            .to_owned()
    }
}

pub fn email_candidate_identifiers_email(email: &str) -> Value {
    json!({ "email": email })
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EmailSummaryContract {
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub risks: Vec<String>,
    pub deadlines: Vec<String>,
    #[serde(default)]
    pub event_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub persona_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub organization_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub document_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub agreement_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub task_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub decision_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub obligation_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub relationship_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub fact_candidates: Vec<EmailKnowledgeCandidate>,
}
