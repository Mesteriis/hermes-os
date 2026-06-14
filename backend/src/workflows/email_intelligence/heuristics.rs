use crate::domains::mail::messages::ProjectedMessage;

const URGENT_WORDS: &[&str] = &[
    "urgent",
    "asap",
    "deadline",
    "immediately",
    "critical",
    "action required",
];
const FINANCE_WORDS: &[&str] = &[
    "invoice",
    "payment",
    "factura",
    "bill",
    "amount due",
    "receipt",
    "tax",
];
const LEGAL_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "legal",
    "liability",
    "confidential",
    "attorney",
];
const ATTACHMENT_WORDS: &[&str] = &["attached", "attachment", "see attached", "please find"];
const JUNK_WORDS: &[&str] = &[
    "unsubscribe",
    "opt out",
    "this email was sent",
    "if you no longer wish",
];

pub(super) fn heuristic_score(message: &ProjectedMessage) -> i16 {
    let mut score: i16 = 30;
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if contains_any(&subject_lower, URGENT_WORDS) {
        score = score.saturating_add(15);
    }
    if contains_any(&body_lower, FINANCE_WORDS) || contains_any(&subject_lower, FINANCE_WORDS) {
        score = score.saturating_add(20);
    }
    if contains_any(&body_lower, LEGAL_WORDS) || contains_any(&subject_lower, LEGAL_WORDS) {
        score = score.saturating_add(25);
    }

    score_question_sign(&mut score, &body_lower);
    score_attachment_language(&mut score, &body_lower);
    score_junk_language(&mut score, &body_lower);

    if message.body_text.len() < 50 {
        score = score.saturating_sub(10);
    }

    score.clamp(0, 100)
}

pub(super) fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
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

fn score_question_sign(score: &mut i16, body_lower: &str) {
    if body_lower.contains('?') {
        *score = score.saturating_add(10);
    }
}

fn score_attachment_language(score: &mut i16, body_lower: &str) {
    if contains_any(body_lower, ATTACHMENT_WORDS) {
        *score = score.saturating_add(10);
    }
}

fn score_junk_language(score: &mut i16, body_lower: &str) {
    if contains_any(body_lower, JUNK_WORDS) {
        *score = score.saturating_sub(20);
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
