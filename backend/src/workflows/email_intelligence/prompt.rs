use crate::domains::mail::messages::ProjectedMessage;

pub(super) const EMAIL_INTELLIGENCE_PROMPT_VERSION: &str = "v1-email-intelligence-2026-06-07";

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
