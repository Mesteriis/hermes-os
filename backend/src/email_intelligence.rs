use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;

use thiserror::Error;

use crate::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, WorkflowState,
};
use crate::ollama::{OllamaClient, OllamaError};

const EMAIL_INTELLIGENCE_PROMPT_VERSION: &str = "v1-email-intelligence-2026-06-07";

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EmailCategory {
    Critical,
    Important,
    Personal,
    Work,
    Finance,
    Legal,
    Notification,
    Newsletter,
    Marketing,
    Spam,
    Scam,
    Phishing,
    Suspicious,
}

impl EmailCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailCategory::Critical => "critical",
            EmailCategory::Important => "important",
            EmailCategory::Personal => "personal",
            EmailCategory::Work => "work",
            EmailCategory::Finance => "finance",
            EmailCategory::Legal => "legal",
            EmailCategory::Notification => "notification",
            EmailCategory::Newsletter => "newsletter",
            EmailCategory::Marketing => "marketing",
            EmailCategory::Spam => "spam",
            EmailCategory::Scam => "scam",
            EmailCategory::Phishing => "phishing",
            EmailCategory::Suspicious => "suspicious",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "critical" => Some(EmailCategory::Critical),
            "important" => Some(EmailCategory::Important),
            "personal" => Some(EmailCategory::Personal),
            "work" => Some(EmailCategory::Work),
            "finance" => Some(EmailCategory::Finance),
            "legal" => Some(EmailCategory::Legal),
            "notification" => Some(EmailCategory::Notification),
            "newsletter" => Some(EmailCategory::Newsletter),
            "marketing" => Some(EmailCategory::Marketing),
            "spam" => Some(EmailCategory::Spam),
            "scam" => Some(EmailCategory::Scam),
            "phishing" => Some(EmailCategory::Phishing),
            "suspicious" => Some(EmailCategory::Suspicious),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct EmailIntelligenceService {
    ollama: Option<OllamaClient>,
}

impl EmailIntelligenceService {
    pub fn new(ollama: Option<OllamaClient>) -> Self {
        Self { ollama }
    }

    pub async fn analyze_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Option<EmailAnalysis>, EmailIntelligenceError> {
        let Some(ref ollama) = self.ollama else {
            return Ok(None);
        };

        let prompt = build_email_analysis_prompt(message);
        let result = ollama.chat(&prompt).await?;

        let json_text = result
            .content
            .trim()
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .map(str::trim)
            .unwrap_or(result.content.trim());

        let mut analysis: EmailAnalysis = serde_json::from_str(json_text)
            .map_err(|e| EmailIntelligenceError::ParseError(e.to_string()))?;

        analysis.model = result.model;
        analysis.prompt_version = EMAIL_INTELLIGENCE_PROMPT_VERSION.to_owned();

        Ok(Some(analysis))
    }

    pub async fn analyze_and_persist(
        &self,
        store: &MessageProjectionStore,
        message: &ProjectedMessage,
    ) -> Result<bool, EmailIntelligenceError> {
        let Some(analysis) = self.analyze_message(message).await? else {
            return Ok(false);
        };

        let workflow_hint = if analysis.is_spam || analysis.is_phishing {
            Some(WorkflowState::Spam)
        } else if analysis.importance_score >= 80 {
            Some(WorkflowState::NeedsAction)
        } else {
            None
        };

        store
            .set_ai_analysis(
                &message.message_id,
                Some(&analysis.category),
                Some(&analysis.summary),
                Some(analysis.importance_score),
            )
            .await?;

        if let Some(state) = workflow_hint {
            let _ = store
                .transition_workflow_state(&message.message_id, state)
                .await;
        }

        Ok(true)
    }

    pub fn heuristic_score(message: &ProjectedMessage) -> i16 {
        let mut score: i16 = 30;
        let body_lower = message.body_text.to_lowercase();
        let subject_lower = message.subject.to_lowercase();

        let urgent_words = [
            "urgent",
            "asap",
            "deadline",
            "immediately",
            "critical",
            "action required",
        ];
        for word in &urgent_words {
            if subject_lower.contains(word) {
                score = score.saturating_add(15);
                break;
            }
        }

        let finance_words = [
            "invoice",
            "payment",
            "factura",
            "bill",
            "amount due",
            "receipt",
            "tax",
        ];
        for word in &finance_words {
            if body_lower.contains(word) || subject_lower.contains(word) {
                score = score.saturating_add(20);
                break;
            }
        }

        let legal_words = [
            "contract",
            "agreement",
            "nda",
            "legal",
            "liability",
            "confidential",
            "attorney",
        ];
        for word in &legal_words {
            if body_lower.contains(word) || subject_lower.contains(word) {
                score = score.saturating_add(25);
                break;
            }
        }

        if body_lower.contains('?') {
            score = score.saturating_add(10);
        }

        let attach_words = ["attached", "attachment", "see attached", "please find"];
        for word in &attach_words {
            if body_lower.contains(word) {
                score = score.saturating_add(10);
                break;
            }
        }

        let junk_words = [
            "unsubscribe",
            "opt out",
            "this email was sent",
            "if you no longer wish",
        ];
        for word in &junk_words {
            if body_lower.contains(word) {
                score = score.saturating_sub(20);
                break;
            }
        }

        if message.body_text.len() < 50 {
            score = score.saturating_sub(10);
        }

        score.clamp(0, 100)
    }

    pub fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
        let body_lower = message.body_text.to_lowercase();
        let subject_lower = message.subject.to_lowercase();

        if body_lower.contains("invoice")
            || body_lower.contains("factura")
            || body_lower.contains("payment")
        {
            return Some("finance".to_owned());
        }
        if body_lower.contains("contract")
            || body_lower.contains("nda")
            || body_lower.contains("agreement")
        {
            return Some("legal".to_owned());
        }
        if body_lower.contains("unsubscribe") || body_lower.contains("newsletter") {
            return Some("marketing".to_owned());
        }
        if subject_lower.contains("notification") || body_lower.contains("notification") {
            return Some("notification".to_owned());
        }
        if body_lower.contains("click here")
            && (body_lower.contains("account") || body_lower.contains("verify"))
        {
            return Some("suspicious".to_owned());
        }

        None
    }
}

fn build_email_analysis_prompt(message: &ProjectedMessage) -> String {
    let body = if message.body_text.len() <= 2000 {
        &message.body_text
    } else {
        &message.body_text[..2000.min(message.body_text.len())]
    };

    format!(
        "You are an email intelligence assistant. Analyze this email and respond with a JSON object containing:\n\
- category: one of [critical, important, personal, work, finance, legal, notification, newsletter, marketing, spam, scam, phishing, suspicious]\n\
- summary: 1-2 sentence TL;DR\n\
- importance_score: integer 0-100\n\
- is_spam: boolean\n\
- is_phishing: boolean\n\
- suggested_action: what the recipient should do, or null\n\
- extracted_deadline: any deadline mentioned, or null\n\
- language: the language code (e.g., \"en\", \"es\", \"ru\"), or null\n\
\n\
From: {}\n\
Subject: {}\n\
Body:\n\
{}\n\
\n\
Respond with ONLY the JSON object.",
        message.sender, message.subject, body
    )
}

#[derive(Debug, Error)]
pub enum EmailIntelligenceError {
    #[error(transparent)]
    Ollama(#[from] OllamaError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error("failed to parse AI response: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::WorkflowState;
    use chrono::Utc;

    fn test_message(subject: &str, body: &str) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "msg:test:1".into(),
            raw_record_id: "raw:1".into(),
            account_id: "acct:1".into(),
            provider_record_id: "prov:1".into(),
            subject: subject.into(),
            sender: "sender@example.com".into(),
            recipients: vec!["recipient@example.com".into()],
            body_text: body.into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: json!({}),
            workflow_state: WorkflowState::New,
            importance_score: None,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
        }
    }

    #[test]
    fn heuristic_score_urgent_subject() {
        let msg = test_message("URGENT: Action Required", "Please respond ASAP");
        let score = EmailIntelligenceService::heuristic_score(&msg);
        assert!(score >= 35, "got {score}");
    }

    #[test]
    fn heuristic_score_finance_body() {
        let msg = test_message(
            "Update",
            "Please find the invoice attached for payment. Amount due: $500",
        );
        let score = EmailIntelligenceService::heuristic_score(&msg);
        assert!(score >= 50, "got {score}");
    }

    #[test]
    fn heuristic_score_marketing_body() {
        let msg = test_message(
            "Digest",
            "Click here. To unsubscribe, click here. If you no longer wish to receive...",
        );
        let score = EmailIntelligenceService::heuristic_score(&msg);
        assert!(score <= 30, "got {score}");
    }

    #[test]
    fn heuristic_category_finance() {
        let msg = test_message("Invoice #123", "Here is your invoice for services");
        assert_eq!(
            EmailIntelligenceService::heuristic_category(&msg).as_deref(),
            Some("finance")
        );
    }

    #[test]
    fn heuristic_category_legal() {
        let msg = test_message("Contract", "Please review the NDA and agreement");
        assert_eq!(
            EmailIntelligenceService::heuristic_category(&msg).as_deref(),
            Some("legal")
        );
    }

    #[test]
    fn heuristic_category_none() {
        let msg = test_message("Hello", "Just checking in");
        assert!(EmailIntelligenceService::heuristic_category(&msg).is_none());
    }

    #[test]
    fn email_category_from_str_all_valid() {
        assert_eq!(
            EmailCategory::parse("critical"),
            Some(EmailCategory::Critical)
        );
        assert_eq!(EmailCategory::parse("spam"), Some(EmailCategory::Spam));
        assert_eq!(
            EmailCategory::parse("finance"),
            Some(EmailCategory::Finance)
        );
    }
}
