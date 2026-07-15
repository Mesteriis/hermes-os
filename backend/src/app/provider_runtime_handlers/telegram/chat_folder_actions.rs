use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use super::chat_actions::{
    TelegramChatActionRequest, TelegramChatLifecycleCommandResponse,
    record_chat_lifecycle_command_with_payload,
};
use super::helpers::{AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed};
use crate::app::api_support::stores::{domain_stores::*, integration_stores::*};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::platform::audit::models::NewApiAuditRecord;

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatFolderReassignRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) target_provider_folder_ids: Vec<i64>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatFolderReassignResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) command_ids: Vec<String>,
    pub(crate) added_provider_folder_ids: Vec<i64>,
    pub(crate) removed_provider_folder_ids: Vec<i64>,
}

fn folder_command_payload(source: &str, provider_folder_id: i64) -> Value {
    json!({
        "source": source,
        "provider_folder_id": provider_folder_id,
    })
}

fn chat_folder_ids(metadata: &Value) -> Vec<i64> {
    let Some(metadata_object) = metadata.as_object() else {
        return Vec::new();
    };
    let folder_ids = metadata_object
        .get("tdlib_chat_positions")
        .and_then(Value::as_object)
        .and_then(|positions| positions.get("folder_ids"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
        .unwrap_or_default();
    if !folder_ids.is_empty() {
        return folder_ids;
    }
    metadata_object
        .get("provider_folder_id")
        .and_then(Value::as_i64)
        .map(|value| vec![value])
        .unwrap_or_default()
}

fn unique_folder_ids(folder_ids: Vec<i64>) -> Vec<i64> {
    let mut unique = Vec::with_capacity(folder_ids.len());
    for folder_id in folder_ids {
        if !unique.contains(&folder_id) {
            unique.push(folder_id);
        }
    }
    unique
}

pub(crate) async fn post_telegram_chat_add_folder(
    State(state): State<AppState>,
    Path((telegram_chat_id, provider_folder_id)): Path<(String, i64)>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.folder_add")
        .await?;
    let payload = folder_command_payload("telegram_chat_lifecycle", provider_folder_id);
    let command_id = record_chat_lifecycle_command_with_payload(
        &state,
        Some(&telegram_chat_id),
        &request,
        "folder_add",
        "provider_write",
        None,
        true,
        payload.clone(),
        payload,
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_add(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            provider_folder_id,
        ))
        .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "folder_add".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_remove_folder(
    State(state): State<AppState>,
    Path((telegram_chat_id, provider_folder_id)): Path<(String, i64)>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.folder_remove")
        .await?;
    let payload = folder_command_payload("telegram_chat_lifecycle", provider_folder_id);
    let command_id = record_chat_lifecycle_command_with_payload(
        &state,
        Some(&telegram_chat_id),
        &request,
        "folder_remove",
        "provider_write",
        None,
        true,
        payload.clone(),
        payload,
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_remove(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            provider_folder_id,
        ))
        .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "folder_remove".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_reassign_folders(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatFolderReassignRequest>,
) -> Result<Json<TelegramChatFolderReassignResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(
        &state,
        &request.account_id,
        "dialogs.folder_reassign",
    )
    .await?;
    let target_provider_folder_ids = unique_folder_ids(request.target_provider_folder_ids);
    if target_provider_folder_ids.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "folder reassignment requires at least one target_provider_folder_id".to_owned(),
        )));
    }

    let chat = telegram_provider_runtime_service(&state)?
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    let current_folder_ids = chat_folder_ids(&chat.metadata);
    let added_provider_folder_ids = target_provider_folder_ids
        .iter()
        .copied()
        .filter(|folder_id| !current_folder_ids.contains(folder_id))
        .collect::<Vec<_>>();
    let removed_provider_folder_ids = current_folder_ids
        .iter()
        .copied()
        .filter(|folder_id| !target_provider_folder_ids.contains(folder_id))
        .collect::<Vec<_>>();

    if added_provider_folder_ids.is_empty() && removed_provider_folder_ids.is_empty() {
        return Ok(Json(TelegramChatFolderReassignResponse {
            telegram_chat_id,
            provider_chat_id: request.provider_chat_id,
            action: "folder_reassign".to_owned(),
            status: "noop".to_owned(),
            command_ids: Vec::new(),
            added_provider_folder_ids,
            removed_provider_folder_ids,
        }));
    }

    let lifecycle_request = TelegramChatActionRequest {
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        last_read_inbox_provider_message_id: None,
    };
    let mut command_ids =
        Vec::with_capacity(added_provider_folder_ids.len() + removed_provider_folder_ids.len());

    for provider_folder_id in &added_provider_folder_ids {
        let payload = folder_command_payload("telegram_chat_folder_reassign", *provider_folder_id);
        command_ids.push(
            record_chat_lifecycle_command_with_payload(
                &state,
                Some(&telegram_chat_id),
                &lifecycle_request,
                "folder_add",
                "provider_write",
                None,
                true,
                payload.clone(),
                payload,
            )
            .await?,
        );
    }
    for provider_folder_id in &removed_provider_folder_ids {
        let payload = folder_command_payload("telegram_chat_folder_reassign", *provider_folder_id);
        command_ids.push(
            record_chat_lifecycle_command_with_payload(
                &state,
                Some(&telegram_chat_id),
                &lifecycle_request,
                "folder_remove",
                "provider_write",
                None,
                true,
                payload.clone(),
                payload,
            )
            .await?,
        );
    }

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_reassign(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            &target_provider_folder_ids,
            &added_provider_folder_ids,
            &removed_provider_folder_ids,
        ))
        .await?;

    Ok(Json(TelegramChatFolderReassignResponse {
        telegram_chat_id,
        provider_chat_id: request.provider_chat_id,
        action: "folder_reassign".to_owned(),
        status: "queued".to_owned(),
        command_ids,
        added_provider_folder_ids,
        removed_provider_folder_ids,
    }))
}
