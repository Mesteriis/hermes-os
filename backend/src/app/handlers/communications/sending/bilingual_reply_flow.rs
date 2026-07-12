use super::super::*;

const MAX_BILINGUAL_REPLY_TEXT_CHARS: usize = 64_000;
const BILINGUAL_REPLY_TONES: [&str; 5] = ["formal", "business", "friendly", "short", "detailed"];

struct AiSignalContext {
    event_kind: &'static str,
    subject: serde_json::Value,
    provenance: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct BilingualReplyFlowRequest {
    pub(super) reply_text_ru: String,
    pub(super) tone: String,
}

#[derive(Serialize)]
pub(crate) struct BilingualReplyFlowResponse {
    pub(super) message_id: String,
    pub(super) subject: String,
    pub(super) tone: String,
    pub(super) reply_language: &'static str,
    pub(super) send_ready: bool,
    pub(super) original: BilingualOriginal,
    pub(super) translation: BilingualTranslationStep,
    pub(super) reply: BilingualReplyDraft,
    pub(super) back_translation: BilingualTranslationStep,
}

#[derive(Serialize)]
pub(crate) struct BilingualOriginal {
    pub(super) language: String,
    pub(super) confidence: f32,
    pub(super) text: String,
}

#[derive(Serialize)]
pub(crate) struct BilingualTranslationStep {
    pub(super) target: String,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct BilingualReplyDraft {
    pub(super) language: &'static str,
    pub(super) tone: String,
    pub(super) text: String,
}

pub(crate) async fn post_v1_bilingual_reply_flow(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<BilingualReplyFlowRequest>,
) -> Result<Json<BilingualReplyFlowResponse>, ApiError> {
    let reply_text = req.reply_text_ru.trim();
    if reply_text.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "reply_text_ru is required",
        ));
    }
    if reply_text.chars().count() > MAX_BILINGUAL_REPLY_TEXT_CHARS {
        return Err(ApiError::InvalidCommunicationQuery(
            "reply_text_ru exceeds maximum length",
        ));
    }

    let tone = normalize_bilingual_reply_tone(&req.tone)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    crate::app::api_support::require_mail_ai_content_egress(
        &state,
        &msg.account_id,
        crate::app::api_support::MailAiContentEgressKind::Body,
    )
    .await?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        );
    let original_language = detection.language.clone();
    let back_translation_target = if original_language == "unknown" {
        "en".to_owned()
    } else {
        original_language.clone()
    };
    let service = email_multilingual_service(&state).await?;

    let translation = bilingual_translation_step_with_signal(
        &state,
        &service,
        &msg.body_text,
        "ru",
        &message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_inbound_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_inbound_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_inbound_translation",
                "message_id": message_id,
            }),
        },
    )
    .await?;
    let back_translation = bilingual_translation_step_with_signal(
        &state,
        &service,
        reply_text,
        &back_translation_target,
        &message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_back_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_back_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_back_translation",
                "message_id": message_id,
            }),
        },
    )
    .await?;
    let send_ready = translation.translated && back_translation.translated;

    Ok(Json(BilingualReplyFlowResponse {
        message_id: msg.message_id,
        subject: reply_subject(&msg.subject),
        tone: tone.clone(),
        reply_language: "ru",
        send_ready,
        original: BilingualOriginal {
            language: original_language,
            confidence: detection.confidence,
            text: msg.body_text,
        },
        translation,
        reply: BilingualReplyDraft {
            language: "ru",
            tone,
            text: reply_text.to_owned(),
        },
        back_translation,
    }))
}

fn normalize_bilingual_reply_tone(value: &str) -> Result<String, ApiError> {
    let tone = value.trim().to_ascii_lowercase();
    if BILINGUAL_REPLY_TONES.contains(&tone.as_str()) {
        return Ok(tone);
    }
    Err(ApiError::InvalidCommunicationQuery(
        "unsupported bilingual reply tone",
    ))
}

async fn bilingual_translation_step(
    state: &AppState,
    service: &crate::domains::communications::multilingual::MultilingualService,
    text: &str,
    target_language: &str,
    message_id: &str,
) -> Result<BilingualTranslationStep, ApiError> {
    bilingual_translation_step_with_signal(
        state,
        service,
        text,
        target_language,
        message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_translation",
                "message_id": message_id,
            }),
        },
    )
    .await
}

async fn bilingual_translation_step_with_signal(
    state: &AppState,
    service: &crate::domains::communications::multilingual::MultilingualService,
    text: &str,
    target_language: &str,
    message_id: &str,
    signal: AiSignalContext,
) -> Result<BilingualTranslationStep, ApiError> {
    match service.translate(text, target_language).await {
        Ok(Some(translation)) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal_best_effort(
                    pool.clone(),
                    signal.event_kind,
                    message_id,
                    signal.subject,
                    json!({
                        "target_language": translation.target_language,
                        "model": translation.model,
                    }),
                    signal.provenance,
                    None,
                )
                .await;
            }

            Ok(BilingualTranslationStep {
                target: translation.target_language,
                translated: true,
                text: Some(translation.translated_text),
                model: Some(translation.model),
                reason: None,
            })
        }
        Ok(None) => Ok(BilingualTranslationStep {
            target: target_language.to_owned(),
            translated: false,
            text: None,
            model: None,
            reason: Some("no LLM configured".to_owned()),
        }),
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message_id,
                target_language = %target_language,
                "bilingual reply translation failed"
            );
            Ok(BilingualTranslationStep {
                target: target_language.to_owned(),
                translated: false,
                text: None,
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
            })
        }
    }
}

fn reply_subject(subject: &str) -> String {
    if subject.trim_start().to_ascii_lowercase().starts_with("re:") {
        subject.to_owned()
    } else {
        format!("Re: {subject}")
    }
}
