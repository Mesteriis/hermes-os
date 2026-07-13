use super::super::*;

const ATTACHMENT_TRANSLATION_SOURCE: &str = "durable_extracted_text";

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
    crate::app::api_support::stores::ai_runtime::require_mail_ai_content_egress(
        &state,
        &msg.account_id,
        crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
    )
    .await?;
    let service = email_multilingual_service(&state).await?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        );
    match service
        .translate(&msg.body_text, &req.target_language)
        .await?
    {
        Some(t) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                    pool.clone(),
                    "message_translation",
                    &message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "translation",
                    }),
                    serde_json::json!({
                        "target_language": t.target_language,
                        "original_language": detection.language,
                        "model": t.model,
                    }),
                    serde_json::json!({
                        "source": "communication_message_translation",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await;
            }

            Ok(Json(
                serde_json::json!({"translated": true, "text": t.translated_text, "target": t.target_language, "model": t.model}),
            ))
        }
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

    let attachment = communication_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if attachment.attachment.scan_status
        != crate::domains::communications::storage::AttachmentSafetyScanStatus::Clean
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment translation requires a clean scan verdict",
        ));
    }
    let message = message_store(&state)?
        .message(&attachment.attachment.message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let source_text = crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionService::new(
        pool,
        crate::domains::communications::storage::LocalCommunicationBlobStore::new(
            crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT,
        ),
    )
    .completed_text(&attachment_id)
    .await
    .map_err(|_| ApiError::InvalidCommunicationQuery("attachment extracted text is unavailable"))?
    .ok_or(ApiError::FailedPrecondition(
        "extract attachment text before translation".to_owned(),
    ))?;
    crate::app::api_support::stores::ai_runtime::require_mail_ai_content_egress(
        &state,
        &message.account_id,
        crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::ExtractedText,
    )
    .await?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &source_text.text,
        );
    let service = email_multilingual_service(&state).await?;

    match service.translate(&source_text.text, target_language).await {
        Ok(Some(translation)) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                    pool.clone(),
                    "attachment_translation",
                    &attachment.attachment.attachment_id,
                    serde_json::json!({
                        "kind": "communication_attachment",
                        "source_code": "ai",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                        "operation": "attachment_translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                        "source": ATTACHMENT_TRANSLATION_SOURCE,
                    }),
                    serde_json::json!({
                        "source": "communication_attachment_translation",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                    }),
                    None,
                )
                .await;
            }

            Ok(Json(AttachmentTranslationResponse {
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
            }))
        }
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
    let thread_store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let messages = thread_store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;
    crate::app::api_support::stores::ai_runtime::require_mail_ai_content_egress(
        &state,
        account_id,
        crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
    )
    .await?;
    let service = email_multilingual_service(&state).await?;
    let mut items = Vec::with_capacity(messages.len());

    for message in messages {
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );
        match service.translate(&message.body_text, target_language).await {
            Ok(Some(translation)) => {
                if let Some(pool) = state.database.pool() {
                    crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                        pool.clone(),
                        "thread_message_translation",
                        &message.message_id,
                        serde_json::json!({
                            "kind": "communication_message",
                            "source_code": "ai",
                            "message_id": message.message_id,
                            "operation": "thread_message_translation",
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        serde_json::json!({
                            "target_language": translation.target_language,
                            "original_language": detection.language,
                            "model": translation.model,
                        }),
                        serde_json::json!({
                            "source": "communication_thread_message_translation",
                            "message_id": message.message_id,
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        None,
                    )
                    .await;
                }

                items.push(ThreadTranslationItem {
                    message_id: message.message_id,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: true,
                    text: Some(translation.translated_text),
                    target: translation.target_language,
                    model: Some(translation.model),
                    reason: None,
                })
            }
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
