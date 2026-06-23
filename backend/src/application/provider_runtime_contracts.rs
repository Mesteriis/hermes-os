use std::sync::Arc;

use sqlx::PgPool;

pub(crate) use crate::integrations::telegram::client::commands::{
    list_commands_filtered, list_commands_filtered as list_telegram_commands_filtered,
};
pub(crate) use crate::integrations::telegram::client::models::messages::{
    TelegramCommandKind, TelegramCommandListResponse, TelegramDeleteRequest, TelegramEditRequest,
    TelegramForwardChainResponse, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessage,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramProviderWriteCommand, TelegramReactionListResponse, TelegramReactionRequest,
    TelegramReactionResponse, TelegramReplyChainResponse, TelegramReplyRequest,
    TelegramRestoreVisibilityRequest,
};
pub(crate) use crate::integrations::telegram::client::topics::{
    get_topic as get_telegram_topic, list_topic_message_ids as list_telegram_topic_message_ids,
    list_topics as list_telegram_topics, search_topics as search_telegram_topics_projection,
};
pub(crate) use crate::integrations::telegram::client::{
    NewTelegramMessage, ProviderCommunicationMessage, TelegramAccount,
    TelegramAccountLifecycleResponse, TelegramAccountListResponse, TelegramAccountSetupRequest,
    TelegramAccountSetupResponse, TelegramAttachmentAnchor, TelegramAttachmentDownloadStateUpdate,
    TelegramChat, TelegramChatGroupFilter, TelegramChatGroupFilterListResponse, TelegramChatMember,
    TelegramError, TelegramLiveAccountSetupRequest, TelegramMessageIngestResult,
    TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
    TelegramSecretVault, TelegramTopic, TelegramTopicCloseRequest, TelegramTopicCreateRequest,
    TelegramTopicLifecycleResponse, TelegramTopicListResponse, ensure_telegram_account_active,
    telegram_chat_id,
};
pub(crate) use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
    TelegramMediaSendType, TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest,
    TelegramRuntimeStatus, TelegramRuntimeStopRequest,
};
pub(crate) mod qr_login {
    pub(crate) use crate::integrations::telegram::tdjson::{
        cancel_qr_login, start_qr_login, submit_qr_login_password,
    };
}
pub(crate) mod lifecycle {
    pub(crate) use crate::integrations::telegram::client::lifecycle::*;
}
pub(crate) mod models {
    pub(crate) use crate::integrations::telegram::client::models::*;

    pub(crate) mod messages {
        pub(crate) use crate::integrations::telegram::client::models::messages::*;
    }
}
pub(crate) use crate::integrations::telegram::client::TelegramStore as TelegramProviderRuntimeStore;
pub(crate) use crate::integrations::whatsapp::client::WhatsappWebStore as WhatsappProviderRuntimeStore;
pub(crate) use crate::integrations::whatsapp::client::{
    NewWhatsappWebMessage, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebError, WhatsappWebMessage, WhatsappWebMessageIngestResult, WhatsappWebSession,
};

pub(crate) fn telegram_provider_runtime_store(pool: PgPool) -> TelegramProviderRuntimeStore {
    TelegramProviderRuntimeStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone()),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool,
            ),
        ),
    )
}

pub(crate) fn whatsapp_provider_runtime_store(pool: PgPool) -> WhatsappProviderRuntimeStore {
    WhatsappProviderRuntimeStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(crate::domains::communications::messages::ProviderChannelMessageStore::new(pool)),
    )
}
