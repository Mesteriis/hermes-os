use super::super::*;

pub(crate) async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
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
    let svc = crate::domains::communications::extract::EmailExtractService::new(
        ai_hub_optional(&state).await?,
    );
    let tasks = svc.extract_tasks(&msg).await?;
    let external_llm_task_count = tasks
        .iter()
        .filter(|task| task.source == "ai_hub.external_llm")
        .count();
    if external_llm_task_count > 0
        && let Some(pool) = state.database.pool()
    {
        crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
            pool.clone(),
            "message_task_extraction",
            &message_id,
            serde_json::json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "task_extraction",
            }),
            serde_json::json!({
                "task_count": tasks.len(),
                "external_llm_task_count": external_llm_task_count,
            }),
            serde_json::json!({
                "source": "communication_message_task_extraction",
                "message_id": message_id,
            }),
            None,
        )
        .await;
    }
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

pub(crate) async fn post_v1_extract_notes(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::communications::extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}
