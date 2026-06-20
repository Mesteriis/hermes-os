use super::super::*;

const MAX_BILINGUAL_REPLY_TEXT_CHARS: usize = 64_000;
const BILINGUAL_REPLY_TONES: [&str; 5] = ["formal", "business", "friendly", "short", "detailed"];

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

    let translation = bilingual_translation_step(&service, &msg.body_text, "ru", &message_id).await;
    let back_translation =
        bilingual_translation_step(&service, reply_text, &back_translation_target, &message_id)
            .await;
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
    service: &crate::domains::communications::multilingual::MultilingualService,
    text: &str,
    target_language: &str,
    message_id: &str,
) -> BilingualTranslationStep {
    match service.translate(text, target_language).await {
        Ok(Some(translation)) => BilingualTranslationStep {
            target: translation.target_language,
            translated: true,
            text: Some(translation.translated_text),
            model: Some(translation.model),
            reason: None,
        },
        Ok(None) => BilingualTranslationStep {
            target: target_language.to_owned(),
            translated: false,
            text: None,
            model: None,
            reason: Some("no LLM configured".to_owned()),
        },
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message_id,
                target_language = %target_language,
                "bilingual reply translation failed"
            );
            BilingualTranslationStep {
                target: target_language.to_owned(),
                translated: false,
                text: None,
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
            }
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
