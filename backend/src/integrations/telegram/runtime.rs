mod actor;
mod commands;
mod manager;
mod models;
mod participant_commands;
mod state;
mod status;
#[cfg(test)]
mod tests;
mod validation;

const TDJSON_BOOTSTRAP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_COMMAND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_RECEIVE_POLL_SECONDS: f64 = 1.0;

pub(crate) use self::manager::TelegramProviderSearchRequest;
pub use self::manager::TelegramRuntimeManager;
pub(crate) use self::manager::command_executor::execute_queued_commands;
pub(crate) use self::manager::{
    TelegramMediaDownloadContext, TelegramMemberSyncContext, TelegramRuntimeEventBridgeContext,
    TelegramRuntimeOperationContext, TelegramRuntimeOperationDeps, TelegramRuntimeStartContext,
};
pub use self::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncMode,
    TelegramHistorySyncRequest, TelegramHistorySyncResponse, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramMediaSendRequest, TelegramRuntimeRestartRequest,
    TelegramRuntimeStartRequest, TelegramRuntimeStatus, TelegramRuntimeStopRequest,
};
