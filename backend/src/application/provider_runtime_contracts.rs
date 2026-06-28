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
pub(crate) use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCallIngestResult, WhatsappWebDeliveryState,
    WhatsappWebDialogIngestResult, WhatsappWebError, WhatsappWebMediaIngestResult,
    WhatsappWebMessage, WhatsappWebMessageDeleteIngestResult, WhatsappWebMessageIngestResult,
    WhatsappWebMessageUpdateIngestResult, WhatsappWebParticipantIngestResult,
    WhatsappWebPresenceIngestResult, WhatsappWebReactionIngestResult,
    WhatsappWebReceiptIngestResult, WhatsappWebRuntimeEventIngestResult, WhatsappWebSession,
    WhatsappWebStatusDeleteIngestResult, WhatsappWebStatusIngestResult,
    WhatsappWebStatusViewIngestResult,
};
pub(crate) use crate::integrations::whatsapp::runtime::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppCommandDeadLetterRequest,
    WhatsAppConversationCommandRequest, WhatsAppCredentialBinding, WhatsAppDeleteRequest,
    WhatsAppEditRequest, WhatsAppForwardRequest, WhatsAppMediaDownloadRequest,
    WhatsAppMediaUploadRequest, WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest,
    WhatsAppProviderCommand, WhatsAppProviderCommandListResponse, WhatsAppProviderCommandResponse,
    WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape, WhatsAppQrLinkSession,
    WhatsAppQrLinkStartRequest, WhatsAppReactionRequest, WhatsAppReplyRequest,
    WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest,
    WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest,
    WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest, WhatsAppStatusPublishRequest,
    WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest,
    whatsapp_business_cloud_access_token_secret_ref, whatsapp_business_cloud_app_secret_ref,
    whatsapp_business_cloud_runtime, whatsapp_business_cloud_webhook_verify_token_ref,
    whatsapp_native_md_runtime, whatsapp_provider_runtime_mux, whatsapp_web_companion_runtime,
};
pub(crate) use crate::integrations::zoom::client::ZoomStore as ZoomProviderRuntimeStore;
pub(crate) use crate::integrations::zoom::client::{
    ZoomAccount, ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomError, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthCompleteRequest,
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomOAuthStartResponse,
    ZoomRecordingImportAuditResponse, ZoomRecordingImportRemoveRequest,
    ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingSyncRequest, ZoomRecordingSyncResult,
    ZoomRetentionCleanupItem, ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse,
    ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest,
    ZoomRuntimeStatus, ZoomRuntimeStopRequest, ZoomServerToServerAuthorizeRequest,
    ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest,
    ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult,
    ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest, ZoomWebhookSubscription,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};

pub(crate) type WhatsAppProviderRuntimeRef = Arc<dyn WhatsAppProviderRuntime>;

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

pub(crate) fn whatsapp_provider_runtime(pool: PgPool) -> WhatsAppProviderRuntimeRef {
    let provider_account_store = Arc::new(
        crate::domains::communications::core::CommunicationProviderAccountStore::new(pool.clone()),
    );
    let provider_secret_binding_store = Arc::new(
        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        ),
    );
    let provider_channel_message_store = Arc::new(
        crate::domains::communications::messages::ProviderChannelMessageStore::new(pool.clone()),
    );
    let whatsapp_runtime_event_sink = Arc::new(
        crate::application::WhatsappRuntimeSignalIngestService::new(pool.clone()),
    );
    let web_companion_runtime = whatsapp_web_companion_runtime(
        pool.clone(),
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
    );
    let native_md_runtime = whatsapp_native_md_runtime(
        pool.clone(),
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
        whatsapp_runtime_event_sink,
    );
    let business_cloud_runtime = whatsapp_business_cloud_runtime(
        pool,
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
    );
    whatsapp_provider_runtime_mux(
        provider_account_store,
        web_companion_runtime,
        native_md_runtime,
        business_cloud_runtime,
    )
}

pub(crate) fn zoom_provider_runtime_store(
    pool: PgPool,
    event_bus: crate::platform::events::EventBus,
) -> ZoomProviderRuntimeStore {
    ZoomProviderRuntimeStore::new(
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
            crate::domains::communications::storage::CommunicationStorageStore::new(pool.clone()),
        ),
        crate::platform::calls::CallIntelligenceStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
        event_bus,
    )
}
