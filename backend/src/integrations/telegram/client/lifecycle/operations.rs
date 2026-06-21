use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use super::message_versions::{
    insert_message_version, latest_message_version, latest_version_number, local_edit_diff,
};
use super::tombstones::insert_tombstone;
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::{
    TelegramCommandKind, TelegramDeleteRequest, TelegramEditRequest, TelegramLifecycleResponse,
    TelegramPinRequest, TelegramRestoreVisibilityRequest,
};

pub async fn record_edit(
    store: &TelegramStore,
    request: &TelegramEditRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let pool = store.pool();
    let now = Utc::now();
    let version_number = latest_version_number(pool, message_id).await? + 1;
    let previous_body = previous_message_body(store, message_id).await?;

    let _version = insert_message_version(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        version_number,
        Some(&request.new_text),
        now,
        None,
        local_edit_diff(previous_body.as_deref(), &request.new_text),
        json!({"event": "local_edit"}),
    )
    .await?;

    let idempotency_key = format!("edit:{}:{}", request.provider_message_id, version_number);
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Edit.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        actor_id,
        json!({"new_text": &request.new_text}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"edit_version": version_number}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "edit".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "version_recorded".to_owned(),
        timestamp: now,
        version_number: Some(version_number),
        tombstone_id: None,
    })
}

async fn previous_message_body(
    store: &TelegramStore,
    message_id: &str,
) -> Result<Option<String>, TelegramError> {
    if let Some(version) = latest_message_version(store.pool(), message_id).await?
        && version.body_text.is_some()
    {
        return Ok(version.body_text);
    }

    Ok(store
        .provider_channel_message_store()
        .body_text(message_id)
        .await?)
}

pub async fn record_delete(
    pool: &PgPool,
    request: &TelegramDeleteRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        &request.reason_class,
        &request.actor_class,
        now,
        None,
        request.is_provider_delete,
        false,
    )
    .await?;

    let idempotency_key = format!(
        "delete:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Delete.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "destructive",
        "confirmed",
        actor_id,
        json!({"reason_class": &request.reason_class, "is_provider_delete": request.is_provider_delete}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "delete".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "tombstone_recorded".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_restore_visibility(
    pool: &PgPool,
    request: &TelegramRestoreVisibilityRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        "unknown",
        "owner",
        now,
        None,
        false,
        true,
    )
    .await?;

    let idempotency_key = format!(
        "restore_visibility:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::RestoreVisibility.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"reason": &request.reason}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "restore_visibility".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "visibility_restored".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_pin_state(
    store: &TelegramStore,
    request: &TelegramPinRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let pool = store.pool();
    let now = Utc::now();
    let message = store.message_by_id(message_id).await?.ok_or_else(|| {
        TelegramError::InvalidRequest(format!("telegram message `{message_id}` was not found"))
    })?;
    store
        .append_message_pin_observation(&message, request.is_pinned, now)
        .await?;

    let command_kind = if request.is_pinned {
        TelegramCommandKind::Pin
    } else {
        TelegramCommandKind::Unpin
    };
    let idempotency_key = format!(
        "{}:{}:{}",
        command_kind.as_str(),
        request.provider_message_id,
        now.timestamp_millis()
    );
    insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        command_kind.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"is_pinned": request.is_pinned}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"message_id": message_id, "is_pinned": request.is_pinned}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: command_kind.as_str().to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: if request.is_pinned {
            "pinned".to_owned()
        } else {
            "unpinned".to_owned()
        },
        timestamp: now,
        version_number: None,
        tombstone_id: None,
    })
}
