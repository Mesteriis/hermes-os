use crate::domains::mail::messages::ProjectedMessage;
use crate::workflows::email_intelligence::models::{EmailKnowledgeCandidate, EmailSummaryContract};

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
const ACTION_WORDS: &[&str] = &[
    "action required",
    "please",
    "review",
    "respond",
    "reply",
    "confirm",
    "send",
    "approve",
    "sign",
];
const RISK_WORDS: &[&str] = &[
    "risk",
    "blocked",
    "blocker",
    "issue",
    "problem",
    "phishing",
    "scam",
    "verify your account",
    "click here",
];
const DEADLINE_WORDS: &[&str] = &[
    "deadline",
    "due",
    "by ",
    "before",
    "today",
    "tomorrow",
    "eod",
    "friday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
];
const EVENT_WORDS: &[&str] = &[
    "meeting",
    "call",
    "demo",
    "appointment",
    "interview",
    "workshop",
    "webinar",
];
const DOCUMENT_WORDS: &[&str] = &[
    "attachment",
    "attached",
    "document",
    "file",
    "pdf",
    "invoice",
    "receipt",
    "msa",
    "sow",
];
const AGREEMENT_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "msa",
    "sow",
    "terms",
    "liability",
    "confidential",
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

pub(super) fn structured_summary(message: &ProjectedMessage) -> EmailSummaryContract {
    let mut key_points = Vec::new();
    push_unique_bounded(&mut key_points, cleaned_phrase(&message.subject), 5);

    let phrases = message_phrases(message);
    for phrase in &phrases {
        if key_points.len() >= 5 {
            break;
        }
        let lower = phrase.to_lowercase();
        if !contains_any(&lower, ACTION_WORDS) {
            push_unique_bounded(&mut key_points, Some(phrase.clone()), 5);
        }
    }

    let mut action_items = Vec::new();
    let mut risks = Vec::new();
    let mut deadlines = Vec::new();

    for phrase in phrases {
        let lower = phrase.to_lowercase();
        if contains_any(&lower, ACTION_WORDS) {
            push_unique_bounded(&mut action_items, Some(phrase.clone()), 5);
        }
        if contains_any(&lower, RISK_WORDS) {
            push_unique_bounded(&mut risks, Some(phrase.clone()), 5);
        }
        if contains_any(&lower, DEADLINE_WORDS) {
            push_unique_bounded(&mut deadlines, Some(phrase), 5);
        }
    }

    let candidate_phrases = phrases_for_candidates(message);
    EmailSummaryContract {
        key_points,
        action_items,
        risks,
        deadlines,
        event_candidates: knowledge_candidates(&candidate_phrases, EVENT_WORDS, 5),
        persona_candidates: persona_candidates(message),
        organization_candidates: organization_candidates(message),
        document_candidates: knowledge_candidates(&candidate_phrases, DOCUMENT_WORDS, 5),
        agreement_candidates: knowledge_candidates(&candidate_phrases, AGREEMENT_WORDS, 5),
    }
}

fn phrases_for_candidates(message: &ProjectedMessage) -> Vec<String> {
    let mut phrases = Vec::new();
    push_unique_bounded(&mut phrases, cleaned_phrase(&message.subject), 30);
    for phrase in message_phrases(message) {
        push_unique_bounded(&mut phrases, Some(phrase), 30);
    }
    phrases
}

fn knowledge_candidates(
    phrases: &[String],
    words: &[&str],
    limit: usize,
) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    for phrase in phrases {
        let lower = phrase.to_lowercase();
        if contains_any(&lower, words) {
            push_candidate_bounded(&mut candidates, phrase.clone(), phrase.clone(), limit);
        }
    }
    candidates
}

fn persona_candidates(message: &ProjectedMessage) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    push_persona_candidate(
        &mut candidates,
        message.sender_display_name.as_deref(),
        &message.sender,
    );
    push_persona_candidate(
        &mut candidates,
        Some(message.sender.as_str()),
        &message.sender,
    );
    for line in message.body_text.lines().take(20) {
        let trimmed = line.trim();
        if let Some((label, email)) = email_identity(trimmed) {
            push_candidate_bounded(&mut candidates, label, email, 5);
        }
    }
    candidates
}

fn organization_candidates(message: &ProjectedMessage) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    let mut values: Vec<&str> = Vec::with_capacity(message.recipients.len() + 16);
    values.push(&message.sender);
    values.extend(message.recipients.iter().map(String::as_str));
    values.extend(message.body_text.split_whitespace().take(80));
    for value in values {
        if let Some(domain) = email_domain(value) {
            push_candidate_bounded(&mut candidates, domain.clone(), value.to_owned(), 5);
        }
    }
    candidates
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

fn message_phrases(message: &ProjectedMessage) -> Vec<String> {
    message
        .body_text
        .split(['.', '\n', '!', '?'])
        .filter_map(cleaned_phrase)
        .take(30)
        .collect()
}

fn cleaned_phrase(value: &str) -> Option<String> {
    let phrase = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches(|c: char| matches!(c, '"' | '\'' | ':' | ';' | ',' | '-' | ' '))
        .to_owned();
    (!phrase.is_empty()).then_some(phrase)
}

fn push_unique_bounded(target: &mut Vec<String>, value: Option<String>, limit: usize) {
    let Some(value) = value else {
        return;
    };
    if target.len() >= limit || target.iter().any(|existing| existing == &value) {
        return;
    }
    target.push(value);
}

fn push_persona_candidate(
    target: &mut Vec<EmailKnowledgeCandidate>,
    label: Option<&str>,
    evidence: &str,
) {
    let Some(label) = label.and_then(cleaned_phrase) else {
        return;
    };
    if label.contains('@') && label.len() > 120 {
        return;
    }
    push_candidate_bounded(target, label, evidence.to_owned(), 5);
}

fn email_identity(value: &str) -> Option<(String, String)> {
    let email = value
        .split_whitespace()
        .find(|part| part.contains('@'))
        .map(|part| {
            part.trim_matches(|c| matches!(c, '<' | '>' | ',' | ';'))
                .to_owned()
        })?;
    let label = value
        .split('<')
        .next()
        .and_then(cleaned_phrase)
        .filter(|name| !name.eq_ignore_ascii_case("from"))
        .unwrap_or_else(|| email.clone());
    Some((label, email))
}

fn email_domain(value: &str) -> Option<String> {
    let email = value
        .trim_matches(|c| matches!(c, '<' | '>' | ',' | ';' | ')' | '(' | '"' | '\''))
        .split('@')
        .nth(1)?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
        .to_lowercase();
    if email.is_empty() || email.ends_with(".local") {
        None
    } else {
        Some(email)
    }
}

fn push_candidate_bounded(
    target: &mut Vec<EmailKnowledgeCandidate>,
    title: String,
    evidence: String,
    limit: usize,
) {
    let title = title.trim().to_owned();
    let evidence = evidence.trim().to_owned();
    if title.is_empty()
        || target.len() >= limit
        || target
            .iter()
            .any(|candidate| candidate.title.eq_ignore_ascii_case(&title))
    {
        return;
    }
    target.push(EmailKnowledgeCandidate { title, evidence });
}
