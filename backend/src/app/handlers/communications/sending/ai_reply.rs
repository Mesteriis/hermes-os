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
    crate::app::api_support::require_mail_ai_content_egress(
        &state,
        &msg.account_id,
        crate::app::api_support::MailAiContentEgressKind::Body,
    )
    .await?;
    let service = email_ai_reply_service(&state).await?;
    let opts = crate::domains::communications::ai_reply::AiReplyOptions {
        tone: req.tone,
        language: req.language,
        context: req.context,
    };
    match service.generate_reply(&msg, &opts).await? {
        Some(draft) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal_best_effort(
                    pool.clone(),
                    "reply_drafting",
                    &message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "reply_drafting",
                    }),
                    serde_json::json!({
                        "tone": draft.tone,
                        "language": draft.language,
                    }),
                    serde_json::json!({
                        "source": "communication_message_ai_reply",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await;
            }

            Ok(Json(
                serde_json::json!({"subject": draft.subject, "body": draft.body, "tone": draft.tone, "language": draft.language}),
            ))
        }
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
    crate::app::api_support::require_mail_ai_content_egress(
        &state,
        &msg.account_id,
        crate::app::api_support::MailAiContentEgressKind::Body,
    )
    .await?;
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
    if !variants.is_empty()
        && let Some(pool) = state.database.pool()
    {
        crate::domains::signal_hub::dispatch_ai_helper_signal_best_effort(
            pool.clone(),
            "reply_variant_generation",
            &message_id,
            serde_json::json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "reply_variant_generation",
            }),
            serde_json::json!({
                "variant_count": variants.len(),
                "language_count": languages.len(),
                "tone_count": tones.len(),
            }),
            serde_json::json!({
                "source": "communication_message_ai_reply_variants",
                "message_id": message_id,
            }),
            None,
        )
        .await;
    }
    Ok(Json(serde_json::json!({"variants": variants})))
}
