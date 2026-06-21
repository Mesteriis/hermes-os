use crate::app::api_support::{TelegramCapabilitiesResponse, event_store};
use crate::app::{ApiError, AppState};
use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::{TelegramStore, telegram_chat_id};
use crate::integrations::telegram::runtime::TelegramRuntimeEventBridgeContext;
use crate::platform::config::AppConfig;
use crate::platform::events::NewEventEnvelope;
use crate::platform::secrets::SecretReferenceStore;
use serde_json::json;

pub(super) const AUDIT_ACTOR_ID: &str = "hermes-frontend";

pub(super) fn telegram_api_hash_from_config(config: &AppConfig) -> Option<String> {
    config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
}

pub(super) fn telegram_secret_store(state: &AppState) -> Result<SecretReferenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(SecretReferenceStore::new(pool.clone()))
}

pub(super) fn telegram_runtime_event_bridge_context(
    state: &AppState,
) -> TelegramRuntimeEventBridgeContext {
    TelegramRuntimeEventBridgeContext::new(state.database.pool().cloned(), state.event_bus.clone())
}

pub(super) async fn publish_telegram_event(
    state: &AppState,
    event: NewEventEnvelope,
) -> Result<(), ApiError> {
    if state.database.pool().is_some()
        && let Err(error) = event_store(state)?.append(&event).await
    {
        tracing::warn!(error = %error, "failed to append event to event store");
    }

    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn ensure_telegram_account_operation_allowed(
    state: &AppState,
    account_id: &str,
    operation: &str,
) -> Result<(), ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let account = CommunicationProviderAccountStore::new(pool)
        .get(account_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram account `{account_id}` is not configured"
            )))
        })?;
    if !account.provider_kind.is_telegram() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        ))));
    }

    let capabilities = TelegramCapabilitiesResponse::current_for_account(&state.config, &account);
    let capability = capabilities
        .capabilities
        .iter()
        .find(|item| item.operation == operation)
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram capability `{operation}` is not defined"
            )))
        })?;

    if matches!(capability.status.as_str(), "available" | "degraded") {
        return Ok(());
    }

    Err(ApiError::Telegram(TelegramError::InvalidRequest(
        capability.reason.clone(),
    )))
}

pub(super) async fn telegram_message_snapshot_payload(
    store: &TelegramStore,
    message_id: &str,
    base_payload: serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let mut payload = match base_payload {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };

    if let Some(message) = store.message_by_id(message_id).await? {
        payload.insert("message".to_owned(), json!(message));
        if let Some(provider_chat_id) = message.provider_chat_id.as_deref() {
            let projected_chat = store
                .telegram_chat(&message.account_id, provider_chat_id)
                .await?;
            let resolved_chat_id = projected_chat
                .as_ref()
                .map(|chat| chat.telegram_chat_id.clone())
                .unwrap_or_else(|| telegram_chat_id(&message.account_id, provider_chat_id));
            payload.insert("telegram_chat_id".to_owned(), json!(resolved_chat_id));
            if let Some(chat) = projected_chat {
                payload.insert("chat".to_owned(), json!(chat));
            }
        }
    }

    Ok(serde_json::Value::Object(payload))
}
