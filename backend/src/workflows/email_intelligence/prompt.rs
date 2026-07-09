use std::borrow::Cow;

use crate::ai::hub::LocalAiInspection;
use crate::domains::communications::messages::ProjectedMessage;

pub(super) const EMAIL_INTELLIGENCE_PROMPT_VERSION: &str =
    "v4-mail-ai-privacy-gated-candidates-2026-07-08";

pub(super) fn build_email_analysis_prompt(
    message: &ProjectedMessage,
    target_language: &str,
    inspection: &LocalAiInspection,
) -> String {
    let body = if message.body_text.chars().count() <= 2000 {
        Cow::Borrowed(message.body_text.as_str())
    } else {
        Cow::Owned(message.body_text.chars().take(2000).collect::<String>())
    };

    format!(
        "You are an email intelligence assistant inside Hermes Hub. Analyze this email and respond with a JSON object containing:\n\
- category: one of [critical, important, personal, work, finance, legal, notification, newsletter, marketing, spam, scam, phishing, suspicious]\n\
- summary: 1-2 sentence TL;DR in target_language when translation is needed\n\
- key_points: array of up to 5 short evidence-backed key points\n\
- action_items: array of up to 5 requested or implied actions\n\
- risks: array of up to 5 risks, blockers, scams, suspicious details or delivery concerns\n\
- deadlines: array of up to 5 deadlines or time constraints\n\
- event_candidates: array of up to 5 candidate objects\n\
- persona_candidates: array of up to 5 candidate objects\n\
- organization_candidates: array of up to 5 candidate objects\n\
- document_candidates: array of up to 5 candidate objects\n\
- agreement_candidates: array of up to 5 candidate objects\n\
- task_candidates: array of up to 5 candidate objects\n\
- decision_candidates: array of up to 5 candidate objects\n\
- obligation_candidates: array of up to 5 candidate objects\n\
- relationship_candidates: array of up to 5 candidate objects\n\
- fact_candidates: array of up to 5 candidate objects\n\
- importance_score: integer 0-100\n\
- is_spam: boolean\n\
- is_phishing: boolean\n\
- suggested_action: what the recipient should do, or null\n\
- extracted_deadline: any deadline mentioned, or null\n\
- language: the language code (e.g., \"en\", \"es\", \"ru\"), or null\n\
\n\
Candidate object shape: {{\"kind\": string, \"title\": string, \"summary\": string, \"evidence\": string, \"confidence\": number, \"identifiers\": object}}.\n\
Do not create persona, organization, task, decision, obligation, relationship or fact candidates when is_spam or is_phishing is true.\n\
Newsletter/marketing that is not spam may include sender/brand persona or organization candidates.\n\
Use evidence from the email only; do not invent durable facts.\n\
\n\
Target language: {}\n\
Local inspection: language={} confidence={} sensitivity={:?}\n\
\n\
From: {}\n\
Subject: {}\n\
Body:\n\
{}\n\
\n\
Respond with ONLY the JSON object.",
        target_language,
        inspection.language.language,
        inspection.language.confidence,
        inspection.sensitivity,
        message.sender,
        message.subject,
        body.as_ref()
    )
}
