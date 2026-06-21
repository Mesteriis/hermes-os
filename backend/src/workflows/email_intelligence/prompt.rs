use crate::domains::communications::messages::ProjectedMessage;

pub(super) const EMAIL_INTELLIGENCE_PROMPT_VERSION: &str =
    "v3-email-intelligence-mail-knowledge-candidates-2026-06-15";

pub(super) fn build_email_analysis_prompt(message: &ProjectedMessage) -> String {
    let body = if message.body_text.len() <= 2000 {
        &message.body_text
    } else {
        &message.body_text[..2000.min(message.body_text.len())]
    };

    format!(
        "You are an email intelligence assistant. Analyze this email and respond with a JSON object containing:\n\
- category: one of [critical, important, personal, work, finance, legal, notification, newsletter, marketing, spam, scam, phishing, suspicious]\n\
- summary: 1-2 sentence TL;DR\n\
- key_points: array of up to 5 short evidence-backed key points\n\
- action_items: array of up to 5 requested or implied actions\n\
- risks: array of up to 5 risks, blockers, scams, suspicious details or delivery concerns\n\
- deadlines: array of up to 5 deadlines or time constraints\n\
- event_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- persona_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- organization_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- document_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- agreement_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
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
