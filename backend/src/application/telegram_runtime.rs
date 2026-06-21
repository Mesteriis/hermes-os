use crate::app::api_support::{
    communication_provider_account_store, communication_provider_secret_binding_store,
    telegram_store,
};
use crate::app::{ApiError, AppState};
use crate::integrations::telegram::client::models::messages::{
    TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};
use crate::integrations::telegram::client::{TelegramChatMember, TelegramError};
use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadContext, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramMemberSyncContext, TelegramProviderSearchRequest,
    TelegramRuntimeEventBridgeContext, TelegramRuntimeOperationContext,
    TelegramRuntimeRestartRequest, TelegramRuntimeStartContext, TelegramRuntimeStartRequest,
    TelegramRuntimeStatus, TelegramRuntimeStopRequest,
};
use crate::platform::secrets::SecretReferenceStore;

fn telegram_secret_store(state: &AppState) -> Result<SecretReferenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(SecretReferenceStore::new(pool.clone()))
}

fn event_bridge_context(state: &AppState) -> TelegramRuntimeEventBridgeContext {
    TelegramRuntimeEventBridgeContext::new(telegram_store(state).ok(), state.event_bus.clone())
}

pub(crate) async fn runtime_status(
    state: &AppState,
    account_id: &str,
) -> Result<TelegramRuntimeStatus, ApiError> {
    Ok(state
        .telegram_runtime
        .status_for_account(
            &communication_provider_account_store(state)?,
            &state.config,
            account_id,
        )
        .await?)
}

pub(crate) async fn start_runtime(
    state: &AppState,
    request: &TelegramRuntimeStartRequest,
) -> Result<TelegramRuntimeStatus, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeStartContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bus: &state.event_bus,
    };

    Ok(state
        .telegram_runtime
        .start_account(&context, request)
        .await?)
}

pub(crate) async fn stop_runtime(
    state: &AppState,
    request: &TelegramRuntimeStopRequest,
) -> Result<TelegramRuntimeStatus, ApiError> {
    Ok(state
        .telegram_runtime
        .stop_account_runtime(
            &communication_provider_account_store(state)?,
            &state.config,
            request,
        )
        .await?)
}

pub(crate) async fn restart_runtime(
    state: &AppState,
    request: &TelegramRuntimeRestartRequest,
) -> Result<TelegramRuntimeStatus, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeStartContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bus: &state.event_bus,
    };

    Ok(state
        .telegram_runtime
        .restart_account_runtime(&context, request)
        .await?)
}

pub(crate) async fn sync_chat_members(
    state: &AppState,
    telegram_chat_id: &str,
) -> Result<Vec<TelegramChatMember>, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    Ok(state
        .telegram_runtime
        .sync_chat_members(
            TelegramMemberSyncContext {
                provider_account_store: &provider_account_store,
                provider_secret_binding_store: &provider_secret_binding_store,
                telegram_store: &store,
                secret_store: &secret_store,
                secret_resolver: &state.vault,
                config: &state.config,
                event_bridge: Some(event_bridge_context(state)),
            },
            telegram_chat_id,
        )
        .await?)
}

pub(crate) async fn sync_chats(
    state: &AppState,
    request: &TelegramChatSyncRequest,
) -> Result<TelegramChatSyncResponse, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    Ok(state.telegram_runtime.sync_chats(&context, request).await?)
}

pub(crate) async fn sync_history(
    state: &AppState,
    request: &TelegramHistorySyncRequest,
) -> Result<TelegramHistorySyncResponse, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    Ok(state
        .telegram_runtime
        .sync_history(&context, request)
        .await?)
}

pub(crate) async fn send_manual_message(
    state: &AppState,
    request: &TelegramManualSendRequest,
) -> Result<TelegramManualSendResponse, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    Ok(state
        .telegram_runtime
        .send_manual_message(&context, request)
        .await?)
}

pub(crate) async fn send_reply_message(
    state: &AppState,
    request: &TelegramReplyRequest,
) -> Result<TelegramManualSendResponse, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    Ok(state
        .telegram_runtime
        .send_reply_message(&context, request)
        .await?)
}

pub(crate) async fn send_forward_message(
    state: &AppState,
    request: &TelegramForwardRequest,
) -> Result<TelegramManualSendResponse, ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    Ok(state
        .telegram_runtime
        .send_forward_message(&context, request)
        .await?)
}

pub(crate) async fn refresh_provider_search(
    state: &AppState,
    account_id: String,
    provider_chat_id: Option<String>,
    query: String,
    limit: i32,
) -> Result<(), ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    state
        .telegram_runtime
        .search_provider_messages(
            &context,
            &TelegramProviderSearchRequest {
                account_id,
                provider_chat_id,
                query,
                limit,
            },
        )
        .await
        .map(|_| ())
        .map_err(ApiError::Telegram)
}

pub(crate) async fn refresh_forum_topics(
    state: &AppState,
    telegram_chat_id: &str,
) -> Result<(), ApiError> {
    let provider_account_store = communication_provider_account_store(state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)?;
    let store = telegram_store(state)?;
    let secret_store = telegram_secret_store(state)?;
    let context = TelegramRuntimeOperationContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(event_bridge_context(state)),
    };
    state
        .telegram_runtime
        .sync_forum_topics(&context, telegram_chat_id)
        .await
        .map(|_| ())
        .map_err(ApiError::Telegram)
}

pub(crate) async fn download_media(
    state: &AppState,
    request: &TelegramMediaDownloadRequest,
) -> Result<TelegramMediaDownloadResponse, TelegramMediaDownloadApplicationError> {
    let provider_account_store = communication_provider_account_store(state)
        .map_err(TelegramMediaDownloadApplicationError::Setup)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(state)
        .map_err(TelegramMediaDownloadApplicationError::Setup)?;
    let store = telegram_store(state).map_err(TelegramMediaDownloadApplicationError::Setup)?;
    let secret_store =
        telegram_secret_store(state).map_err(TelegramMediaDownloadApplicationError::Setup)?;
    state
        .telegram_runtime
        .download_media(
            TelegramMediaDownloadContext {
                provider_account_store: &provider_account_store,
                provider_secret_binding_store: &provider_secret_binding_store,
                telegram_store: &store,
                secret_store: &secret_store,
                secret_resolver: &state.vault,
                config: &state.config,
                event_bridge: Some(event_bridge_context(state)),
            },
            request,
        )
        .await
        .map_err(TelegramMediaDownloadApplicationError::Runtime)
}

pub(crate) enum TelegramMediaDownloadApplicationError {
    Setup(ApiError),
    Runtime(TelegramError),
}

impl From<TelegramMediaDownloadApplicationError> for ApiError {
    fn from(error: TelegramMediaDownloadApplicationError) -> Self {
        match error {
            TelegramMediaDownloadApplicationError::Setup(error) => error,
            TelegramMediaDownloadApplicationError::Runtime(error) => ApiError::Telegram(error),
        }
    }
}
