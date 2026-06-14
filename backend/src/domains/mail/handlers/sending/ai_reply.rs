use super::super::*;

#[derive(Deserialize)]
pub(crate) struct AiReplyRequest {
    pub(super) tone: Option<String>,
    pub(super) language: Option<String>,
    pub(super) context: Option<String>,
}

pub(crate) async fn post_v1_ai_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let opts = crate::domains::mail::ai_reply::AiReplyOptions {
        tone: req.tone,
        language: req.language,
        context: req.context,
    };
    match service.generate_reply(&msg, &opts).await? {
        Some(draft) => Ok(Json(
            serde_json::json!({"subject": draft.subject, "body": draft.body, "tone": draft.tone, "language": draft.language}),
        )),
        None => Ok(Json(
            serde_json::json!({"generated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyVariantsRequest {
    pub(super) languages: Option<Vec<String>>,
    pub(super) tones: Option<Vec<String>>,
}

pub(crate) async fn post_v1_ai_reply_variants(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyVariantsRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let languages = req
        .languages
        .unwrap_or_else(|| vec!["en".into(), "es".into(), "ru".into()]);
    let tones = req
        .tones
        .unwrap_or_else(|| vec!["professional".into(), "friendly".into()]);
    let variants = service
        .generate_reply_variants(&msg, &languages, &tones)
        .await?;
    Ok(Json(serde_json::json!({"variants": variants})))
}
