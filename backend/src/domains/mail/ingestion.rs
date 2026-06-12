use crate::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, WorkflowState,
};
use crate::workflows::email_intelligence::EmailIntelligenceService;

/// Result of Hermes auto-analysis on an ingested message.
#[derive(Debug)]
pub struct IngestionAnalysis {
    pub category: Option<String>,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub auto_workflow_state: WorkflowState,
}

/// Analyze an incoming message and persist results.
/// This is the mandatory analysis step for every ingested email —
/// provider spam classification is completely ignored.
pub async fn analyze_ingested_message(
    store: &MessageProjectionStore,
    message: &ProjectedMessage,
) -> Result<IngestionAnalysis, MessageProjectionError> {
    let score = EmailIntelligenceService::heuristic_score(message);
    let category = EmailIntelligenceService::heuristic_category(message);

    let body_lower = message.body_text.to_lowercase();

    let is_spam = body_lower.contains("unsubscribe")
        && (body_lower.contains("buy now")
            || body_lower.contains("limited offer")
            || body_lower.contains("click here"));
    let is_phishing = (body_lower.contains("verify your account")
        || body_lower.contains("confirm your password")
        || body_lower.contains("urgent action required"))
        && !message.sender.contains(&message.account_id);

    let auto_state = if is_phishing || (is_spam && score < 20) {
        WorkflowState::Spam
    } else if score >= 75 {
        WorkflowState::NeedsAction
    } else {
        WorkflowState::New
    };

    store
        .set_ai_analysis(&message.message_id, category.as_deref(), None, Some(score))
        .await?;

    if auto_state != WorkflowState::New {
        store
            .transition_workflow_state(&message.message_id, auto_state)
            .await?;
    }

    Ok(IngestionAnalysis {
        category,
        importance_score: score,
        is_spam,
        is_phishing,
        auto_workflow_state: auto_state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::mail::messages::LocalMessageState;
    use chrono::Utc;
    use serde_json::json;

    fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "m:1".into(),
            raw_record_id: "r:1".into(),
            account_id: "personal@ex.com".into(),
            provider_record_id: "p:1".into(),
            subject: subject.into(),
            sender: sender.into(),
            recipients: vec!["me@ex.com".into()],
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
            local_state: LocalMessageState::Active,
            local_state_changed_at: None,
            local_state_reason: None,
        }
    }

    #[test]
    fn phishing_detection_flags_spam() {
        let msg = test_message(
            "Urgent",
            "hacker@evil.com",
            "Please verify your account immediately by clicking here",
        );
        let analysis = EmailIntelligenceService::heuristic_score(&msg);
        assert!(analysis > 0);
    }

    #[test]
    fn newsletter_detected_as_low_score() {
        let msg = test_message(
            "Weekly Digest",
            "news@company.com",
            "Click here to read. To unsubscribe, click here.",
        );
        let score = EmailIntelligenceService::heuristic_score(&msg);
        assert!(score <= 30, "newsletters should score low, got {score}");
    }

    #[test]
    fn invoice_gets_high_score() {
        let msg = test_message(
            "Invoice #456",
            "billing@vendor.com",
            "Please find your invoice attached. Amount due: $500. Payment required by June 15.",
        );
        let score = EmailIntelligenceService::heuristic_score(&msg);
        assert!(score >= 50, "invoices should score high, got {score}");
    }
}
