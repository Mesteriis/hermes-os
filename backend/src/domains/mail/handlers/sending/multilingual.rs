use super::super::*;

pub(crate) async fn get_v1_detect_language(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::mail::multilingual::LanguageDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::mail::multilingual::MultilingualService::detect_language(&msg.body_text),
    ))
}

#[derive(Deserialize)]
pub(crate) struct TranslateRequest {
    pub(super) target_language: String,
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
