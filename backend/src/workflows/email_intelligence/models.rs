use serde::{Deserialize, Serialize};

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
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub suggested_action: Option<String>,
    pub extracted_deadline: Option<String>,
    pub language: Option<String>,
    pub model: String,
    pub prompt_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailKnowledgeCandidate {
    pub title: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
}
