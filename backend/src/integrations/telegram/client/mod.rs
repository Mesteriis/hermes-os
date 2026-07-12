mod accounts;
mod chat_metadata;
mod chat_reconciliation;
mod chat_state;
mod chats;
pub mod commands;
mod errors;
mod evidence;
mod identifiers;
pub mod lifecycle;
mod messages;
pub mod models;
mod observations;
pub mod participants;
mod reactions;
mod references;
pub mod rows;
mod search;
mod store;
#[cfg(test)]
mod tests;
pub mod topics;
mod validation;
mod vault;

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";
const TELEGRAM_CHAT_RECORD_KIND: &str = "telegram_chat";
const TELEGRAM_ACCOUNT_ACTIVE: &str = "active";
const TELEGRAM_ACCOUNT_LOGGED_OUT: &str = "logged_out";
const TELEGRAM_ACCOUNT_REMOVED: &str = "removed";

pub use self::chat_state::{
    TelegramProviderChatPositionUpdate, reconcile_archive_commands_from_provider_state,
    reconcile_folder_add_commands_from_provider_state,
    reconcile_folder_remove_commands_from_provider_state,
    reconcile_mark_read_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
pub use self::errors::TelegramError;
pub(crate) use self::messages::TelegramAttachmentDownloadStateUpdate;
pub(in crate::integrations::telegram) use self::messages::reaction_metadata::derive_tdlib_chosen_reaction_emojis;
pub(in crate::integrations::telegram) use self::messages::reaction_metadata::{
    derive_tdlib_provider_reactions, derive_tdlib_reaction_summary_metadata,
};
pub use self::models::messages::TelegramReactionRequest;
pub use self::models::{
    NewTelegramChat, NewTelegramChatParticipant, NewTelegramMessage, NewTelegramTopic,
    TelegramAccount, TelegramAccountLifecycleResponse, TelegramAccountListResponse,
    TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramChatGroupFilter, TelegramChatGroupFilterListResponse, TelegramChatKind,
    TelegramChatMember, TelegramCredentialBinding, TelegramDeliveryState, TelegramForwardRequest,
    TelegramLiveAccountSetupRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramMessage, TelegramMessageIngestResult, TelegramQrLoginPasswordRequest,
    TelegramQrLoginStartRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
    TelegramReplyRequest, TelegramSyncState, TelegramTopic, TelegramTopicCloseRequest,
    TelegramTopicCreateRequest, TelegramTopicLifecycleResponse, TelegramTopicListResponse,
};
pub use self::participants::{
    mark_absent_members_from_exhaustive_roster, telegram_self_provider_member_id,
};
pub(in crate::integrations::telegram) use self::reactions::{
    TelegramReactionMessageRef, sync_provider_reactions,
};
pub use self::reactions::{add_reaction, reconcile_reaction_commands_from_provider_reactions};
pub use self::store::TelegramStore;
pub use self::vault::TelegramSecretVault;
pub type ProviderCommunicationMessage = TelegramMessage;

pub(crate) use self::identifiers::{
    ensure_telegram_account_active, telegram_chat_id, telegram_text_preview_hash,
};
pub(crate) use self::models::TelegramAttachmentAnchor;
