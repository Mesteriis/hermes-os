use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailAnalysis {
    pub category: String,
    pub summary: String,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub suggested_action: Option<String>,
    pub extracted_deadline: Option<String>,
    pub language: Option<String>,
    pub model: String,
    pub prompt_version: String,
}
