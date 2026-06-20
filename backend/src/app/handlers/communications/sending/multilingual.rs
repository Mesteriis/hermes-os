use super::super::*;

const MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS: usize = 64_000;
const ATTACHMENT_TRANSLATION_SOURCE: &str = "caller_provided_extracted_text";

pub(crate) async fn get_v1_detect_language(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::communications::multilingual::LanguageDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        ),
    ))
}

#[derive(Deserialize)]
pub(crate) struct TranslateRequest {
    pub(super) target_language: String,
}

#[derive(Deserialize)]
pub(crate) struct TranslateAttachmentRequest {
    pub(super) target_language: String,
    pub(super) source_text: String,
}

#[derive(Deserialize)]
pub(crate) struct TranslateThreadQuery {
    pub(super) account_id: String,
    pub(super) subject: String,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadTranslationResponse {
    pub(super) account_id: String,
    pub(super) subject: String,
    pub(super) target_language: String,
    pub(super) items: Vec<ThreadTranslationItem>,
}

#[derive(Serialize)]
pub(crate) struct ThreadTranslationItem {
    pub(super) message_id: String,
    pub(super) original_language: String,
    pub(super) confidence: f32,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) target: String,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AttachmentTranslationResponse {
    pub(super) attachment_id: String,
    pub(super) message_id: String,
    pub(super) filename: Option<String>,
    pub(super) original_language: String,
    pub(super) confidence: f32,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) target: String,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) source: &'static str,
}

pub(crate) async fn post_v1_translate(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_multilingual_service(&state).await?;
    match service
        .translate(&msg.body_text, &req.target_language)
        .await?
    {
        Some(t) => Ok(Json(
            serde_json::json!({"translated": true, "text": t.translated_text, "target": t.target_language, "model": t.model}),
        )),
        None => Ok(Json(
            serde_json::json!({"translated": false, "reason": "no LLM configured"}),
        )),
    }
}

pub(crate) async fn post_v1_translate_attachment(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
    Json(req): Json<TranslateAttachmentRequest>,
) -> Result<Json<AttachmentTranslationResponse>, ApiError> {
    let target_language = req.target_language.trim();
    if target_language.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "target_language is required",
        ));
    }

    let source_text = req.source_text.trim();
    if source_text.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "source_text is required",
        ));
    }
    if source_text.chars().count() > MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS {
        return Err(ApiError::InvalidCommunicationQuery(
            "source_text exceeds maximum length",
        ));
    }

    let attachment = mail_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            source_text,
        );
    let service = email_multilingual_service(&state).await?;

    match service.translate(source_text, target_language).await {
        Ok(Some(translation)) => Ok(Json(AttachmentTranslationResponse {
            attachment_id: attachment.attachment.attachment_id,
            message_id: attachment.attachment.message_id,
            filename: attachment.attachment.filename,
            original_language: detection.language,
            confidence: detection.confidence,
            translated: true,
            text: Some(translation.translated_text),
            target: translation.target_language,
            model: Some(translation.model),
            reason: None,
            source: ATTACHMENT_TRANSLATION_SOURCE,
        })),
        Ok(None) => Ok(Json(AttachmentTranslationResponse {
            attachment_id: attachment.attachment.attachment_id,
            message_id: attachment.attachment.message_id,
            filename: attachment.attachment.filename,
            original_language: detection.language,
            confidence: detection.confidence,
            translated: false,
            text: None,
            target: target_language.to_owned(),
            model: None,
            reason: Some("no LLM configured".to_owned()),
            source: ATTACHMENT_TRANSLATION_SOURCE,
        })),
        Err(error) => {
            tracing::warn!(
                error = %error,
                attachment_id = %attachment.attachment.attachment_id,
                "attachment translation failed"
            );
            Ok(Json(AttachmentTranslationResponse {
                attachment_id: attachment.attachment.attachment_id,
                message_id: attachment.attachment.message_id,
                filename: attachment.attachment.filename,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: false,
                text: None,
                target: target_language.to_owned(),
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
                source: ATTACHMENT_TRANSLATION_SOURCE,
            }))
        }
    }
}

pub(crate) async fn post_v1_translate_thread(
    State(state): State<AppState>,
    Query(query): Query<TranslateThreadQuery>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<ThreadTranslationResponse>, ApiError> {
    let account_id = query.account_id.trim();
    let subject = query.subject.trim();
    let target_language = req.target_language.trim();
    if account_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ));
    }
    if subject.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("subject is required"));
    }
    if target_language.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "target_language is required",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let thread_store = crate::domains::communications::threads::CommunicationThreadStore::new(pool);
    let messages = thread_store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;
    let service = email_multilingual_service(&state).await?;
    let mut items = Vec::with_capacity(messages.len());

    for message in messages {
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );
        match service.translate(&message.body_text, target_language).await {
            Ok(Some(translation)) => items.push(ThreadTranslationItem {
                message_id: message.message_id,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: true,
                text: Some(translation.translated_text),
                target: translation.target_language,
                model: Some(translation.model),
                reason: None,
            }),
            Ok(None) => items.push(ThreadTranslationItem {
                message_id: message.message_id,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: false,
                text: None,
                target: target_language.to_owned(),
                model: None,
                reason: Some("no LLM configured".to_owned()),
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %message.message_id,
                    "thread message translation failed"
                );
                items.push(ThreadTranslationItem {
                    message_id: message.message_id,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: false,
                    text: None,
                    target: target_language.to_owned(),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                });
            }
        }
    }

    Ok(Json(ThreadTranslationResponse {
        account_id: account_id.to_owned(),
        subject: subject.to_owned(),
        target_language: target_language.to_owned(),
        items,
    }))
}
