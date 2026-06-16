mod accounts;
mod chats;
mod errors;
mod identifiers;
pub mod lifecycle;
mod messages;
pub mod models;
mod projection;
pub mod rows;
mod search;
mod store;
#[cfg(test)]
mod tests;
mod validation;
mod vault;

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";
const TELEGRAM_CHAT_RECORD_KIND: &str = "telegram_chat";
const TELEGRAM_ACCOUNT_ACTIVE: &str = "active";
const TELEGRAM_ACCOUNT_LOGGED_OUT: &str = "logged_out";
const TELEGRAM_ACCOUNT_REMOVED: &str = "removed";

pub use self::errors::TelegramError;
pub use self::models::{
    NewTelegramChat, NewTelegramMessage, TelegramAccount, TelegramAccountLifecycleResponse,
    TelegramAccountListResponse, TelegramAccountSetupRequest, TelegramAccountSetupResponse,
    TelegramChat, TelegramChatGroupFilter, TelegramChatGroupFilterListResponse, TelegramChatKind,
    TelegramChatMember, TelegramCredentialBinding, TelegramDeliveryState,
    TelegramLiveAccountSetupRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramMessage, TelegramMessageIngestResult, TelegramQrLoginPasswordRequest,
    TelegramQrLoginStartRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
    TelegramSyncState,
};
pub use self::projection::project_raw_telegram_message;
pub use self::store::TelegramStore;
pub use self::vault::TelegramSecretVault;

pub(crate) use self::identifiers::{
    ensure_telegram_account_active, telegram_chat_id, telegram_text_preview_hash,
};
pub(crate) use self::models::TelegramAttachmentAnchor;
