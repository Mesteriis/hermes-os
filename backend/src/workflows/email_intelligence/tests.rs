use super::*;
use crate::domains::mail::messages::{LocalMessageState, ProjectedMessage, WorkflowState};
use chrono::Utc;
use serde_json::json;

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
        local_state: LocalMessageState::Active,
        local_state_changed_at: None,
        local_state_reason: None,
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
