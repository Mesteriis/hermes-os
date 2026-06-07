use crate::domains::mail::messages::ProjectedMessage;

pub struct WhyImportantContext {
    pub reasons: Vec<String>,
}

pub fn explain_importance(message: &ProjectedMessage) -> WhyImportantContext {
    let mut reasons = Vec::new();
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if let Some(score) = message.importance_score {
        reasons.push(format!("importance score is {score}/100"));
    }

    if subject_lower.contains("urgent") || subject_lower.contains("asap") {
        reasons.push("subject contains urgency markers".into());
    }

    let finance_words = ["invoice", "payment", "factura", "amount", "tax", "bill"];
    for w in &finance_words {
        if body_lower.contains(w) || subject_lower.contains(w) {
            reasons.push("contains financial information".into());
            break;
        }
    }

    let legal_words = ["contract", "nda", "agreement", "legal", "liability"];
    for w in &legal_words {
        if body_lower.contains(w) || subject_lower.contains(w) {
            reasons.push("contains legal or contractual content".into());
            break;
        }
    }

    if body_lower.contains('?') {
        reasons.push("contains a question (likely requires reply)".into());
    }

    if body_lower.contains("deadline")
        || body_lower.contains("due date")
        || body_lower.contains("due by")
    {
        reasons.push("mentions a deadline".into());
    }

    let attach_hints = ["attached", "attachment", "see attached", "please find"];
    for hint in &attach_hints {
        if body_lower.contains(hint) {
            reasons.push("references an attachment".into());
            break;
        }
    }

    let junk_hints = ["unsubscribe", "newsletter", "marketing", "promotion"];
    for hint in &junk_hints {
        if body_lower.contains(hint) || subject_lower.contains(hint) {
            reasons.push("appears to be a newsletter or marketing email".into());
            break;
        }
    }

    if reasons.is_empty() {
        reasons.push("no specific importance signals detected".into());
    }

    WhyImportantContext { reasons }
}

pub fn smart_cc_suggestions(message: &ProjectedMessage) -> Vec<String> {
    let mut suggestions = Vec::new();
    let body_lower = message.body_text.to_lowercase();

    if body_lower.contains("invoice")
        || body_lower.contains("factura")
        || body_lower.contains("payment")
    {
        suggestions.push("Consider adding your accountant/bookkeeper to CC".into());
    }
    if body_lower.contains("contract") || body_lower.contains("legal") || body_lower.contains("nda")
    {
        suggestions.push("Consider adding legal counsel to CC".into());
    }
    if body_lower.contains("project")
        && (body_lower.contains("update") || body_lower.contains("status"))
    {
        suggestions.push("Consider adding project stakeholders to CC".into());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::mail::messages::WorkflowState;
    use chrono::Utc;

    fn test_message(subject: &str, body: &str, score: Option<i16>) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "msg:1".into(),
            raw_record_id: "raw:1".into(),
            account_id: "a:1".into(),
            provider_record_id: "p:1".into(),
            subject: subject.into(),
            sender: "s@e.com".into(),
            recipients: vec!["r@e.com".into()],
            body_text: body.into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: serde_json::json!({}),
            workflow_state: WorkflowState::New,
            importance_score: score,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
        }
    }

    #[test]
    fn explain_importance_urgent_email() {
        let msg = test_message("URGENT: Need response", "Please reply ASAP", Some(80));
        let ctx = explain_importance(&msg);
        assert!(ctx.reasons.iter().any(|r| r.contains("urgency")));
        assert!(ctx.reasons.iter().any(|r| r.contains("80")));
    }

    #[test]
    fn explain_importance_finance_email() {
        let msg = test_message(
            "Invoice attached",
            "Here is the invoice for payment",
            Some(70),
        );
        let ctx = explain_importance(&msg);
        assert!(ctx.reasons.iter().any(|r| r.contains("financial")));
    }

    #[test]
    fn smart_cc_for_invoice() {
        let msg = test_message("Invoice", "Please process this invoice for payment", None);
        let suggestions = smart_cc_suggestions(&msg);
        assert!(suggestions.iter().any(|s| s.contains("accountant")));
    }

    #[test]
    fn smart_cc_for_legal() {
        let msg = test_message("Contract", "Please review the NDA and legal terms", None);
        let suggestions = smart_cc_suggestions(&msg);
        assert!(suggestions.iter().any(|s| s.contains("legal counsel")));
    }
}
