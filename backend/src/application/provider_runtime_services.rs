use hermes_communications_api::evidence::NewRawCommunicationRecord;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::integrations::telegram::client::commands::list_commands_filtered as list_telegram_commands_filtered;
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramCommandListResponse, TelegramMessage, TelegramProviderWriteCommand,
};
use crate::integrations::telegram::client::topics::{
    get_topic as get_telegram_topic, list_topic_message_ids as list_telegram_topic_message_ids,
    list_topics as list_telegram_topics, search_topics as search_telegram_topics_projection,
};
use crate::integrations::telegram::client::{
    NewTelegramMessage, ProviderCommunicationMessage, TelegramAccount, TelegramAccountSetupRequest,
    TelegramAccountSetupResponse, TelegramAttachmentDownloadStateUpdate, TelegramChat,
    TelegramChatMember, TelegramError, TelegramLiveAccountSetupRequest,
    TelegramMessageIngestResult, TelegramSecretVault,
    TelegramStore as TelegramProviderRuntimeStore, TelegramTopic, TelegramTopicLifecycleResponse,
    TelegramTopicListResponse, ensure_telegram_account_active, telegram_chat_id,
};
use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramHistorySyncRequest, TelegramMediaDownloadRequest,
    TelegramMediaSendType, TelegramRuntimeStatus,
};
use crate::integrations::whatsapp::client::{
    WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebError, WhatsappWebMessage, WhatsappWebSession,
};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppConversationCommandRequest,
    WhatsAppCredentialBinding, WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppForwardRequest,
    WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest, WhatsAppPairCodeSession,
    WhatsAppPairCodeStartRequest, WhatsAppProviderCommand, WhatsAppProviderCommandListResponse,
    WhatsAppProviderCommandResponse, WhatsAppProviderRuntimeShape, WhatsAppQrLinkSession,
    WhatsAppQrLinkStartRequest, WhatsAppReactionRequest, WhatsAppReplyRequest,
    WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest,
    WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest,
    WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest, WhatsAppStatusPublishRequest,
    WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest,
};
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::yandex_telemost::client::models::{
    YandexTelemostAccountListResponse, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse,
};
use crate::integrations::yandex_telemost::client::store::YandexTelemostStore;
use crate::integrations::zoom::client::errors::ZoomError;
use crate::integrations::zoom::client::models::{
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthPendingGrant,
    ZoomOAuthStartRequest, ZoomRecordingImportAuditResponse, ZoomRecordingImportRemoveRequest,
    ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingSyncRequest, ZoomRecordingSyncResult,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult,
    ZoomTokenRefreshRequest, ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest,
    ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};
use crate::integrations::zoom::client::store::ZoomStore as ZoomProviderRuntimeStore;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::HostVault;
use hermes_communications_api::accounts::ProviderAccount;

pub(crate) type WhatsAppProviderRuntimeRef =
    Arc<dyn crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderRuntime>;

#[derive(Clone)]
pub(crate) struct TelegramProviderRuntimeApplicationService {
    store: TelegramProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct ZoomProviderRuntimeApplicationService {
    store: ZoomProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct YandexTelemostProviderRuntimeApplicationService {
    store: YandexTelemostStore,
}

impl ZoomProviderRuntimeApplicationService {
    pub(crate) fn new(store: ZoomProviderRuntimeStore) -> Self {
        Self { store }
    }

    pub(crate) async fn setup_fixture_account(
        &self,
        request: &ZoomAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        self.store.setup_fixture_account(request).await
    }

    pub(crate) async fn setup_live_blocked_account(
        &self,
        request: &ZoomLiveAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        self.store.setup_live_blocked_account(request).await
    }

    pub(crate) async fn start_oauth(
        &self,
        request: &ZoomOAuthStartRequest,
    ) -> Result<ZoomOAuthPendingGrant, ZoomError> {
        self.store.start_oauth(request).await
    }

    pub(crate) async fn complete_oauth(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        pending: ZoomOAuthPendingGrant,
        authorization_code: &str,
        external_account_id: Option<&str>,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        self.store
            .complete_oauth(
                secret_store,
                vault,
                pending,
                authorization_code,
                external_account_id,
            )
            .await
    }

    pub(crate) async fn authorize_server_to_server(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomServerToServerAuthorizeRequest,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        self.store
            .authorize_server_to_server(secret_store, vault, request)
            .await
    }

    pub(crate) async fn refresh_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenRefreshRequest,
    ) -> Result<ZoomTokenRefreshResult, ZoomError> {
        self.store.refresh_token(secret_store, vault, request).await
    }

    pub(crate) async fn maintain_tokens(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenMaintenanceRequest,
    ) -> Result<ZoomTokenMaintenanceResult, ZoomError> {
        self.store
            .maintain_tokens(secret_store, vault, request)
            .await
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<ZoomAccountListResponse, ZoomError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn runtime_status(
        &self,
        account_id: &str,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.runtime_status(account_id).await
    }

    pub(crate) async fn start_runtime(
        &self,
        request: &ZoomRuntimeStartRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.start_runtime(request).await
    }

    pub(crate) async fn stop_runtime(
        &self,
        request: &ZoomRuntimeStopRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.stop_runtime(request).await
    }

    pub(crate) async fn remove_runtime(
        &self,
        request: &ZoomRuntimeRemoveRequest,
    ) -> Result<ZoomRuntimeRemoveResponse, ZoomError> {
        self.store.remove_runtime(request).await
    }

    pub(crate) async fn observe_meeting(
        &self,
        request: &ZoomMeetingObservationRequest,
    ) -> Result<ZoomMeetingIngestResult, ZoomError> {
        self.store.observe_meeting(request).await
    }

    pub(crate) async fn observe_recording(
        &self,
        request: &ZoomRecordingObservationRequest,
    ) -> Result<ZoomRecordingIngestResult, ZoomError> {
        self.store.observe_recording(request).await
    }

    pub(crate) async fn import_recording_media_download(
        &self,
        request: &ZoomRecordingMediaDownloadRequest,
        bearer_token: Option<&str>,
    ) -> Result<ZoomRecordingMediaImportResult, ZoomError> {
        self.store
            .import_recording_media_download(request, bearer_token)
            .await
    }

    pub(crate) async fn list_recording_imports(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomRecordingImportAuditResponse, ZoomError> {
        self.store.list_recording_imports(account_id, limit).await
    }

    pub(crate) async fn remove_recording_import(
        &self,
        account_id: &str,
        attachment_id: &str,
        request: &ZoomRecordingImportRemoveRequest,
    ) -> Result<ZoomRecordingImportRemoveResponse, ZoomError> {
        self.store
            .remove_recording_import(account_id, attachment_id, request)
            .await
    }

    pub(crate) async fn list_audit_events(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomAuditEventResponse, ZoomError> {
        self.store.list_audit_events(account_id, limit).await
    }

    pub(crate) async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &ZoomRetentionCleanupRequest,
    ) -> Result<ZoomRetentionCleanupResponse, ZoomError> {
        self.store.cleanup_retention(account_id, request).await
    }

    pub(crate) async fn observe_transcript(
        &self,
        request: &ZoomTranscriptObservationRequest,
    ) -> Result<ZoomTranscriptIngestResult, ZoomError> {
        self.store.observe_transcript(request).await
    }

    pub(crate) async fn import_transcript_file(
        &self,
        request: &ZoomTranscriptFileImportRequest,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        self.store.import_transcript_file(request).await
    }

    pub(crate) async fn import_transcript_file_download(
        &self,
        request: &ZoomTranscriptFileImportRequest,
        download_url: &str,
        download_token: Option<&str>,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        self.store
            .import_transcript_file_download(request, download_url, download_token)
            .await
    }

    pub(crate) async fn sync_recordings(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomRecordingSyncRequest,
        allow_remote_recording_downloads: bool,
        allow_remote_transcript_downloads: bool,
    ) -> Result<ZoomRecordingSyncResult, ZoomError> {
        self.store
            .sync_recordings(
                secret_store,
                vault,
                request,
                allow_remote_recording_downloads,
                allow_remote_transcript_downloads,
            )
            .await
    }

    pub(crate) async fn webhook_subscription_status(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionStatusRequest,
    ) -> Result<ZoomWebhookSubscriptionStatusResult, ZoomError> {
        self.store
            .webhook_subscription_status(secret_store, vault, request)
            .await
    }

    pub(crate) async fn reconcile_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionReconcileRequest,
    ) -> Result<ZoomWebhookSubscriptionReconcileResult, ZoomError> {
        self.store
            .reconcile_webhook_subscription(secret_store, vault, request)
            .await
    }

    pub(crate) async fn remove_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionRemoveRequest,
    ) -> Result<ZoomWebhookSubscriptionRemoveResult, ZoomError> {
        self.store
            .remove_webhook_subscription(secret_store, vault, request)
            .await
    }
}

impl YandexTelemostProviderRuntimeApplicationService {
    pub(crate) fn new(store: YandexTelemostStore) -> Self {
        Self { store }
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<YandexTelemostAccountListResponse, YandexTelemostError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &YandexTelemostRetentionCleanupRequest,
    ) -> Result<YandexTelemostRetentionCleanupResponse, YandexTelemostError> {
        self.store.cleanup_retention(account_id, request).await
    }
}

impl TelegramProviderRuntimeApplicationService {
    pub(crate) fn new(store: TelegramProviderRuntimeStore) -> Self {
        Self { store }
    }

    pub(crate) async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        self.store.setup_fixture_account(request).await
    }

    pub(crate) async fn setup_live_blocked_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        self.store
            .setup_live_blocked_account(secret_store, vault, request)
            .await
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<Vec<TelegramAccount>, TelegramError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn telegram_account_record(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        self.store.telegram_account_record(account_id).await
    }

    pub(crate) async fn logout_account(
        &self,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramError> {
        self.store.logout_account(account_id).await
    }

    pub(crate) async fn remove_account(
        &self,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramError> {
        self.store.remove_account(account_id).await
    }

    pub(crate) async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        self.store.list_chats(account_id, limit).await
    }

    pub(crate) async fn list_chat_group_filters(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<crate::integrations::telegram::client::TelegramChatGroupFilter>, TelegramError>
    {
        self.store.list_chat_group_filters(account_id).await
    }

    pub(crate) async fn telegram_chat_by_id(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        self.store.telegram_chat_by_id(telegram_chat_id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn apply_local_telegram_chat_avatar(
        &self,
        telegram_chat_id: &str,
        tdlib_file_id: i64,
        remote_unique_id: Option<&str>,
        blob_id: &str,
        content_type: &str,
        size_bytes: i64,
        sha256: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        self.store
            .apply_local_chat_avatar(
                telegram_chat_id,
                tdlib_file_id,
                remote_unique_id,
                blob_id,
                content_type,
                size_bytes,
                sha256,
            )
            .await
    }

    pub(crate) async fn list_chat_members(
        &self,
        telegram_chat_id: &str,
        query: Option<&str>,
        role: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<Vec<TelegramChatMember>, TelegramError> {
        self.store
            .list_chat_members(telegram_chat_id, query, role, limit, cursor)
            .await
    }

    pub(crate) async fn search_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store
            .search_messages(account_id, provider_chat_id, query, limit)
            .await
    }

    pub(crate) async fn search_chats(
        &self,
        account_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        self.store.search_chats(account_id, query, limit).await
    }

    pub(crate) async fn pinned_messages(
        &self,
        conversation_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store.pinned_messages(conversation_id, limit).await
    }

    pub(crate) async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store
            .recent_messages(account_id, provider_chat_id, limit)
            .await
    }

    pub(crate) async fn messages_by_ids(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        self.store.messages_by_ids(message_ids).await
    }

    pub(crate) async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        self.store.message_by_id(message_id).await
    }

    pub(crate) async fn set_chat_metadata_bool(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: bool,
    ) -> Result<Value, TelegramError> {
        self.store
            .set_chat_metadata_bool(telegram_chat_id, key, value)
            .await
    }

    pub(crate) async fn set_chat_last_read_at(
        &self,
        telegram_chat_id: &str,
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<Value, TelegramError> {
        self.store
            .set_chat_last_read_at(telegram_chat_id, last_read_at)
            .await
    }

    pub(crate) async fn recompute_chat_unread_count(
        &self,
        telegram_chat_id: &str,
    ) -> Result<Value, TelegramError> {
        self.store
            .recompute_chat_unread_count(telegram_chat_id)
            .await
    }

    pub(crate) async fn list_commands(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        provider_message_id: Option<&str>,
        command_kinds: &[String],
        limit: i64,
    ) -> Result<TelegramCommandListResponse, TelegramError> {
        let items = list_telegram_commands_filtered(
            self.store.pool(),
            account_id,
            provider_chat_id,
            provider_message_id,
            command_kinds,
            limit,
        )
        .await?;
        Ok(TelegramCommandListResponse { items })
    }

    pub(crate) async fn list_topics(
        &self,
        telegram_chat_id: &str,
        limit: i64,
    ) -> Result<TelegramTopicListResponse, TelegramError> {
        let items = list_telegram_topics(self.store.pool(), telegram_chat_id, limit).await?;
        Ok(TelegramTopicListResponse {
            telegram_chat_id: telegram_chat_id.to_owned(),
            items,
        })
    }

    pub(crate) async fn get_topic(
        &self,
        topic_id: &str,
    ) -> Result<Option<TelegramTopic>, TelegramError> {
        get_telegram_topic(self.store.pool(), topic_id).await
    }

    pub(crate) async fn list_topic_message_ids(
        &self,
        topic_id: &str,
        limit: i64,
    ) -> Result<Vec<String>, TelegramError> {
        list_telegram_topic_message_ids(&self.store, topic_id, limit).await
    }

    pub(crate) async fn search_topics(
        &self,
        telegram_chat_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramTopic>, TelegramError> {
        search_telegram_topics_projection(self.store.pool(), telegram_chat_id, query, limit).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn insert_command(
        &self,
        command_id: &str,
        account_id: &str,
        command_kind: &str,
        idempotency_key: &str,
        provider_chat_id: &str,
        provider_message_id: Option<&str>,
        capability_state: &str,
        action_class: &str,
        confirmation_decision: &str,
        actor_id: &str,
        payload: Value,
        target_ref: Value,
        audit_metadata: Value,
    ) -> Result<TelegramProviderWriteCommand, TelegramError> {
        lifecycle::insert_command(
            self.store.pool(),
            command_id,
            account_id,
            command_kind,
            idempotency_key,
            provider_chat_id,
            provider_message_id,
            capability_state,
            action_class,
            confirmation_decision,
            actor_id,
            payload,
            target_ref,
            audit_metadata,
        )
        .await
    }

    pub(crate) async fn find_command_by_idempotency(
        &self,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
        lifecycle::find_command_by_idempotency(self.store.pool(), account_id, idempotency_key).await
    }

    pub(crate) async fn manual_retry_command(
        &self,
        command_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
        lifecycle::manual_retry_command(self.store.pool(), command_id, now).await
    }

    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<crate::integrations::telegram::client::TelegramAttachmentAnchor, TelegramError>
    {
        self.store
            .attachment_anchor_for_message(account_id, provider_chat_id, provider_message_id)
            .await
    }

    pub(crate) async fn update_message_attachment_download_state(
        &self,
        update: TelegramAttachmentDownloadStateUpdate<'_>,
    ) -> Result<(), TelegramError> {
        self.store
            .update_message_attachment_download_state(update)
            .await
    }

    pub(crate) async fn telegram_message_snapshot_payload(
        &self,
        message_id: &str,
        base_payload: Value,
    ) -> Result<Value, TelegramError> {
        crate::application::communication_fixture_ingest::telegram_message_snapshot_payload(
            &self.store,
            message_id,
            base_payload,
        )
        .await
    }
}

#[derive(Clone)]
pub(crate) struct WhatsappProviderRuntimeApplicationService {
    runtime: WhatsAppProviderRuntimeRef,
}

impl WhatsappProviderRuntimeApplicationService {
    pub(crate) fn new(runtime: WhatsAppProviderRuntimeRef) -> Self {
        Self { runtime }
    }

    pub(crate) fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        self.runtime.provider_shape()
    }

    pub(crate) async fn runtime_status(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        self.runtime
            .runtime_status(secret_store, vault, account_id)
            .await
    }

    pub(crate) async fn start_runtime(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeStartRequest,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        self.runtime
            .start_runtime(secret_store, vault, request)
            .await
    }

    pub(crate) async fn stop_runtime(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeStopRequest,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        self.runtime
            .stop_runtime(secret_store, vault, request)
            .await
    }

    pub(crate) async fn revoke_runtime(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeRevokeRequest,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        self.runtime
            .revoke_runtime(secret_store, vault, request)
            .await
    }

    pub(crate) async fn relink_runtime(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeRelinkRequest,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        self.runtime
            .relink_runtime(secret_store, vault, request)
            .await
    }

    pub(crate) async fn remove_runtime(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeRemoveRequest,
    ) -> Result<WhatsAppRuntimeRemoveResponse, WhatsappWebError> {
        self.runtime
            .remove_runtime(secret_store, vault, request)
            .await
    }

    pub(crate) async fn runtime_health(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<WhatsAppRuntimeHealth, WhatsappWebError> {
        self.runtime
            .runtime_health(secret_store, vault, account_id)
            .await
    }

    pub(crate) async fn start_qr_link(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppQrLinkStartRequest,
    ) -> Result<WhatsAppQrLinkSession, WhatsappWebError> {
        self.runtime
            .start_qr_link(secret_store, vault, request)
            .await
    }

    pub(crate) async fn start_pair_code_link(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppPairCodeStartRequest,
    ) -> Result<WhatsAppPairCodeSession, WhatsappWebError> {
        self.runtime
            .start_pair_code_link(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_send_text(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppTextSendRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_send_text(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_reply(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppReplyRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_reply(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_forward(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppForwardRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_forward(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_edit(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppEditRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_edit(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_delete(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppDeleteRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_delete(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_react(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppReactionRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_react(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_unreact(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppReactionRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_unreact(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_media_upload(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppMediaUploadRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_media_upload(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_media_download(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppMediaDownloadRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_media_download(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_mark_read(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_mark_read(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_mark_unread(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_mark_unread(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_archive(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_archive(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_unarchive(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_unarchive(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_mute(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_mute(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_unmute(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_unmute(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_pin(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime.request_pin(secret_store, vault, request).await
    }

    pub(crate) async fn request_unpin(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_unpin(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_join_group(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_join_group(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_leave_group(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppConversationCommandRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_leave_group(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_publish_status(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppStatusPublishRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_publish_status(secret_store, vault, request)
            .await
    }

    pub(crate) async fn request_send_voice_note(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppVoiceNoteSendRequest,
    ) -> Result<WhatsAppProviderCommandResponse, WhatsappWebError> {
        self.runtime
            .request_send_voice_note(secret_store, vault, request)
            .await
    }

    pub(crate) async fn list_provider_commands(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        provider_message_id: Option<&str>,
        command_kinds: &[String],
        limit: i64,
    ) -> Result<WhatsAppProviderCommandListResponse, WhatsappWebError> {
        self.runtime
            .list_provider_commands(
                account_id,
                provider_chat_id,
                provider_message_id,
                command_kinds,
                limit,
            )
            .await
    }

    pub(crate) async fn manual_retry_provider_command(
        &self,
        command_id: &str,
    ) -> Result<Option<WhatsAppProviderCommand>, WhatsappWebError> {
        self.runtime.manual_retry_provider_command(command_id).await
    }

    pub(crate) async fn dead_letter_provider_command(
        &self,
        command_id: &str,
        reason: &str,
    ) -> Result<Option<WhatsAppProviderCommand>, WhatsappWebError> {
        self.runtime
            .dead_letter_provider_command(command_id, reason)
            .await
    }

    pub(crate) async fn store_authorized_session_credential(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        credential: &WhatsAppAuthorizedSessionCredentialWrite,
    ) -> Result<WhatsAppCredentialBinding, WhatsappWebError> {
        self.runtime
            .store_authorized_session_credential(secret_store, vault, credential)
            .await
    }

    pub(crate) async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        self.runtime.setup_fixture_account(request).await
    }

    pub(crate) async fn setup_live_blocked_account(
        &self,
        request: &WhatsappLiveAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        self.runtime.setup_live_blocked_account(request).await
    }

    pub(crate) async fn list_sessions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebSession>, WhatsappWebError> {
        self.runtime.list_sessions(account_id, limit).await
    }

    pub(crate) async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebMessage>, WhatsappWebError> {
        self.runtime
            .recent_messages(account_id, provider_chat_id, limit)
            .await
    }
}

pub(crate) fn telegram_provider_runtime_service(
    pool: PgPool,
) -> TelegramProviderRuntimeApplicationService {
    TelegramProviderRuntimeApplicationService::new(telegram_provider_runtime_store(pool))
}

pub(crate) fn whatsapp_provider_runtime_service(
    pool: PgPool,
) -> WhatsappProviderRuntimeApplicationService {
    WhatsappProviderRuntimeApplicationService::new(whatsapp_provider_runtime(pool))
}

pub(crate) fn zoom_provider_runtime_service(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> ZoomProviderRuntimeApplicationService {
    ZoomProviderRuntimeApplicationService::new(zoom_provider_runtime_store(pool, event_bus))
}

pub(crate) fn yandex_telemost_provider_runtime_service(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> YandexTelemostProviderRuntimeApplicationService {
    YandexTelemostProviderRuntimeApplicationService::new(YandexTelemostStore::new(
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        hermes_events_postgres::store::EventStore::new(pool),
        event_bus,
    ))
}

pub(crate) fn telegram_provider_runtime_store(
    pool: PgPool,
) -> crate::integrations::telegram::client::TelegramStore {
    crate::integrations::telegram::client::TelegramStore::new(
        pool.clone(),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
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
        hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        ),
    );
    let provider_secret_binding_store = Arc::new(
        hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        ),
    );
    let provider_channel_message_store = Arc::new(
        crate::domains::communications::messages::ProviderChannelMessageStore::new(pool.clone()),
    );
    let whatsapp_runtime_event_sink = Arc::new(
        crate::application::whatsapp_runtime_signal_ingest::WhatsappRuntimeSignalIngestService::new(
            pool.clone(),
        ),
    );
    let web_companion_runtime =
        crate::integrations::whatsapp::runtime::whatsapp_web_companion_runtime(
            pool.clone(),
            provider_account_store.clone(),
            provider_secret_binding_store.clone(),
            provider_channel_message_store.clone(),
        );
    let native_md_runtime = crate::integrations::whatsapp::runtime::whatsapp_native_md_runtime(
        pool.clone(),
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
        whatsapp_runtime_event_sink,
    );
    let business_cloud_runtime =
        crate::integrations::whatsapp::runtime::whatsapp_business_cloud_runtime(
            pool,
            provider_account_store.clone(),
            provider_secret_binding_store.clone(),
            provider_channel_message_store.clone(),
        );
    crate::integrations::whatsapp::runtime::whatsapp_provider_runtime_mux(
        provider_account_store,
        web_companion_runtime,
        native_md_runtime,
        business_cloud_runtime,
    )
}

pub(crate) fn zoom_provider_runtime_store(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> crate::integrations::zoom::client::store::ZoomStore {
    crate::integrations::zoom::client::store::ZoomStore::new(
        pool.clone(),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::storage::CommunicationStorageStore::new(pool.clone()),
        ),
        crate::platform::calls::CallIntelligenceStore::new(pool.clone()),
        hermes_events_postgres::store::EventStore::new(pool),
        event_bus,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::integrations::whatsapp::client::{
        NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
        NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
        NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
        NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
        NewWhatsappWebStatusView, WhatsappWebDeliveryState, WhatsappWebLinkState,
        WhatsappWebObservedCall, WhatsappWebObservedDialog, WhatsappWebObservedMedia,
        WhatsappWebObservedMessage, WhatsappWebObservedMessageDelete,
        WhatsappWebObservedMessageUpdate, WhatsappWebObservedParticipant,
        WhatsappWebObservedPresence, WhatsappWebObservedReaction, WhatsappWebObservedReceipt,
        WhatsappWebObservedRuntimeEvent, WhatsappWebObservedStatus,
        WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView,
    };
    use crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderRuntime;
    use crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderRuntimeFuture;
    use crate::platform::secrets::{SecretKind, SecretStoreKind};
    use hermes_communications_api::accounts::CommunicationProviderKind;

    #[derive(Default)]
    struct FakeWhatsAppProviderRuntime {
        calls: Mutex<Vec<&'static str>>,
    }

    impl FakeWhatsAppProviderRuntime {
        fn calls(&self) -> Vec<&'static str> {
            self.calls.lock().expect("calls lock").clone()
        }

        fn record_call(&self, call: &'static str) {
            self.calls.lock().expect("calls lock").push(call);
        }
    }

    impl WhatsAppProviderRuntime for FakeWhatsAppProviderRuntime {
        fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
            WhatsAppProviderRuntimeShape::BusinessCloud
        }

        fn runtime_status<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            account_id: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
            Box::pin(async move {
                self.record_call("runtime_status");
                Ok(fake_runtime_status(account_id))
            })
        }

        fn start_runtime<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppRuntimeStartRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
            Box::pin(async move {
                self.record_call("start_runtime");
                let mut status = fake_runtime_status(&request.account_id);
                status.status = "running".to_owned();
                Ok(status)
            })
        }

        fn stop_runtime<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppRuntimeStopRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
            Box::pin(async move {
                self.record_call("stop_runtime");
                Ok(fake_runtime_status(&request.account_id))
            })
        }

        fn revoke_runtime<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppRuntimeRevokeRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
            Box::pin(async move {
                self.record_call("revoke_runtime");
                let mut status = fake_runtime_status(&request.account_id);
                status.status = "revoked".to_owned();
                status.session_restore_available = false;
                status.session_secret_ref = None;
                status.runtime_blockers = vec!["whatsapp_session_revoked".to_owned()];
                status.last_error =
                    Some("WhatsApp session was revoked and must be relinked".to_owned());
                Ok(status)
            })
        }

        fn relink_runtime<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppRuntimeRelinkRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
            Box::pin(async move {
                self.record_call("relink_runtime");
                let mut status = fake_runtime_status(&request.account_id);
                status.status = "link_required".to_owned();
                status.session_restore_available = false;
                status.session_secret_ref = None;
                status.runtime_blockers = vec!["whatsapp_session_link_required".to_owned()];
                Ok(status)
            })
        }

        fn remove_runtime<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppRuntimeRemoveRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
            Box::pin(async move {
                self.record_call("remove_runtime");
                Ok(WhatsAppRuntimeRemoveResponse {
                    account_id: request.account_id.clone(),
                    provider_kind: "whatsapp_web".to_owned(),
                    removed: true,
                    unbound_secret_refs: Vec::new(),
                    removed_at: Utc::now(),
                })
            })
        }

        fn runtime_health<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            account_id: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
            Box::pin(async move {
                self.record_call("runtime_health");
                Ok(WhatsAppRuntimeHealth {
                    account_id: account_id.to_owned(),
                    provider_shape: "whatsapp_business_cloud".to_owned(),
                    runtime_kind: "fake_business_cloud".to_owned(),
                    status: "available".to_owned(),
                    healthy: true,
                    checks: json!({}),
                    checked_at: Utc::now(),
                })
            })
        }

        fn start_qr_link<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppQrLinkStartRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppQrLinkSession> {
            Box::pin(async move {
                self.record_call("start_qr_link");
                Ok(WhatsAppQrLinkSession {
                    account_id: request.account_id.clone(),
                    provider_shape: "whatsapp_business_cloud".to_owned(),
                    runtime_kind: "fake_business_cloud".to_owned(),
                    status: "blocked".to_owned(),
                    setup_id: "qr-fake".to_owned(),
                    qr_svg: None,
                    expires_at: None,
                    runtime_blockers: vec!["fake_qr_blocked".to_owned()],
                })
            })
        }

        fn start_pair_code_link<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppPairCodeStartRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppPairCodeSession> {
            Box::pin(async move {
                self.record_call("start_pair_code_link");
                Ok(WhatsAppPairCodeSession {
                    account_id: request.account_id.clone(),
                    provider_shape: "whatsapp_business_cloud".to_owned(),
                    runtime_kind: "fake_business_cloud".to_owned(),
                    status: "blocked".to_owned(),
                    setup_id: "pair-fake".to_owned(),
                    phone_number: request.phone_number.clone(),
                    pair_code: None,
                    expires_at: None,
                    runtime_blockers: vec!["fake_pair_code_blocked".to_owned()],
                })
            })
        }

        fn request_send_text<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppTextSendRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_send_text");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-send"),
                    &request.idempotency_key,
                    "send_text",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_reply<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppReplyRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_reply");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-reply"),
                    &request.idempotency_key,
                    "reply",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.reply_to_provider_message_id),
                ))
            })
        }

        fn request_forward<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppForwardRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_forward");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-forward"),
                    &request.idempotency_key,
                    "forward",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.from_provider_message_id),
                ))
            })
        }

        fn request_edit<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppEditRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_edit");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-edit"),
                    &request.idempotency_key,
                    "edit",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.provider_message_id),
                ))
            })
        }

        fn request_delete<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppDeleteRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_delete");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-delete"),
                    &request.idempotency_key,
                    "delete",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.provider_message_id),
                ))
            })
        }

        fn request_react<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppReactionRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_react");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-react"),
                    &request.idempotency_key,
                    "react",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.provider_message_id),
                ))
            })
        }

        fn request_unreact<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppReactionRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_unreact");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-unreact"),
                    &request.idempotency_key,
                    "unreact",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.provider_message_id),
                ))
            })
        }

        fn request_media_upload<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppMediaUploadRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_media_upload");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-media-upload"),
                    &request.idempotency_key,
                    "send_media",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_media_download<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppMediaDownloadRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_media_download");
                Ok(fake_command_response(
                    request
                        .command_id
                        .as_deref()
                        .unwrap_or("fake-media-download"),
                    &request.idempotency_key,
                    "download_media",
                    &request.account_id,
                    &request.provider_chat_id,
                    Some(&request.provider_message_id),
                ))
            })
        }

        fn request_mark_read<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_mark_read");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-mark-read"),
                    &request.idempotency_key,
                    "mark_read",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_mark_unread<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_mark_unread");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-mark-unread"),
                    &request.idempotency_key,
                    "mark_unread",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_archive<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_archive");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-archive"),
                    &request.idempotency_key,
                    "archive",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_unarchive<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_unarchive");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-unarchive"),
                    &request.idempotency_key,
                    "unarchive",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_mute<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_mute");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-mute"),
                    &request.idempotency_key,
                    "mute",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_unmute<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_unmute");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-unmute"),
                    &request.idempotency_key,
                    "unmute",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_pin<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_pin");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-pin"),
                    &request.idempotency_key,
                    "pin",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_unpin<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_unpin");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-unpin"),
                    &request.idempotency_key,
                    "unpin",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_join_group<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_join_group");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-join-group"),
                    &request.idempotency_key,
                    "join_group",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_leave_group<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppConversationCommandRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_leave_group");
                Ok(fake_command_response(
                    request.command_id.as_deref().unwrap_or("fake-leave-group"),
                    &request.idempotency_key,
                    "leave_group",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn request_publish_status<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppStatusPublishRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_publish_status");
                Ok(fake_command_response(
                    request
                        .command_id
                        .as_deref()
                        .unwrap_or("fake-publish-status"),
                    &request.idempotency_key,
                    "publish_status",
                    &request.account_id,
                    "status-feed",
                    None,
                ))
            })
        }

        fn request_send_voice_note<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            request: &'a WhatsAppVoiceNoteSendRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                self.record_call("request_send_voice_note");
                Ok(fake_command_response(
                    request
                        .command_id
                        .as_deref()
                        .unwrap_or("fake-send-voice-note"),
                    &request.idempotency_key,
                    "send_voice_note",
                    &request.account_id,
                    &request.provider_chat_id,
                    None,
                ))
            })
        }

        fn list_provider_commands<'a>(
            &'a self,
            account_id: &'a str,
            provider_chat_id: Option<&'a str>,
            provider_message_id: Option<&'a str>,
            _command_kinds: &'a [String],
            _limit: i64,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse> {
            Box::pin(async move {
                self.record_call("list_provider_commands");
                Ok(WhatsAppProviderCommandListResponse {
                    items: vec![fake_provider_command(
                        "fake-command",
                        "fake-idempotency",
                        "send_text",
                        account_id,
                        provider_chat_id.unwrap_or("fake-chat"),
                        provider_message_id,
                        "cancelled",
                    )],
                })
            })
        }

        fn manual_retry_provider_command<'a>(
            &'a self,
            command_id: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("manual_retry_provider_command");
                Ok(Some(fake_provider_command(
                    command_id,
                    "fake-idempotency",
                    "send_text",
                    "account-1",
                    "fake-chat",
                    None,
                    "retrying",
                )))
            })
        }

        fn dead_letter_provider_command<'a>(
            &'a self,
            command_id: &'a str,
            reason: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("dead_letter_provider_command");
                let mut command = fake_provider_command(
                    command_id,
                    "fake-idempotency",
                    "send_text",
                    "account-1",
                    "fake-chat",
                    None,
                    "dead_letter",
                );
                command.last_error = Some(reason.to_owned());
                Ok(Some(command))
            })
        }

        fn store_authorized_session_credential<'a>(
            &'a self,
            _secret_store: &'a SecretReferenceStore,
            _vault: &'a HostVault,
            credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding> {
            Box::pin(async move {
                self.record_call("store_authorized_session_credential");
                Ok(WhatsAppCredentialBinding {
                    secret_purpose: "whatsapp_web_session_key".to_owned(),
                    secret_ref: format!(
                        "secret:provider-account:{}:whatsapp_web_session_key",
                        credential.account_id
                    ),
                    secret_kind: SecretKind::Other,
                    store_kind: SecretStoreKind::HostVault,
                })
            })
        }

        fn setup_fixture_account<'a>(
            &'a self,
            request: &'a WhatsappWebAccountSetupRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
            Box::pin(async move {
                self.record_call("setup_fixture_account");
                Ok(WhatsappWebAccountSetupResponse {
                    account_id: request.account_id.clone(),
                    provider_kind: request.provider_kind.as_str().to_owned(),
                    runtime: "fake_business_cloud".to_owned(),
                    session: fake_session(&request.account_id),
                })
            })
        }

        fn setup_live_blocked_account<'a>(
            &'a self,
            request: &'a WhatsappLiveAccountSetupRequest,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
            Box::pin(async move {
                self.record_call("setup_live_blocked_account");
                Ok(WhatsappWebAccountSetupResponse {
                    account_id: request.account_id.clone(),
                    provider_kind: request.provider_kind.as_str().to_owned(),
                    runtime: "fake_business_cloud".to_owned(),
                    session: fake_session(&request.account_id),
                })
            })
        }

        fn list_sessions<'a>(
            &'a self,
            account_id: Option<&'a str>,
            _limit: i64,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>> {
            Box::pin(async move {
                self.record_call("list_sessions");
                Ok(vec![fake_session(account_id.unwrap_or("all-accounts"))])
            })
        }

        fn recent_messages<'a>(
            &'a self,
            account_id: Option<&'a str>,
            provider_chat_id: Option<&'a str>,
            _limit: i64,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>> {
            Box::pin(async move {
                self.record_call("recent_messages");
                Ok(vec![WhatsappWebMessage {
                    message_id: "message-1".to_owned(),
                    raw_record_id: "raw-1".to_owned(),
                    account_id: account_id.unwrap_or("account-1").to_owned(),
                    provider_message_id: "provider-message-1".to_owned(),
                    provider_chat_id: provider_chat_id.map(str::to_owned),
                    chat_title: "Runtime boundary".to_owned(),
                    sender: "sender-1".to_owned(),
                    sender_display_name: Some("Sender".to_owned()),
                    text: "hello from fake runtime".to_owned(),
                    occurred_at: Some(Utc::now()),
                    projected_at: Utc::now(),
                    channel_kind: "whatsapp_web".to_owned(),
                    delivery_state: WhatsappWebDeliveryState::Received.as_str().to_owned(),
                    metadata: json!({}),
                }])
            })
        }

        fn ingest_fixture_message<'a>(
            &'a self,
            message: &'a NewWhatsappWebMessage,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessage> {
            Box::pin(async move {
                self.record_call("ingest_fixture_message");
                Ok(WhatsappWebObservedMessage {
                    raw: NewRawCommunicationRecord::new(
                        "raw-fake",
                        &message.account_id,
                        "whatsapp_message",
                        &message.provider_message_id,
                        message.source_fingerprint(),
                        &message.import_batch_id,
                        json!({"text": message.text}),
                    ),
                })
            })
        }

        fn reconcile_fixture_message_commands<'a>(
            &'a self,
            _message: &'a NewWhatsappWebMessage,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_message_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_reaction<'a>(
            &'a self,
            reaction: &'a NewWhatsappWebReaction,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReaction> {
            Box::pin(async move {
                self.record_call("ingest_fixture_reaction");
                Ok(WhatsappWebObservedReaction {
                    raw: NewRawCommunicationRecord::new(
                        "raw-reaction-fake",
                        &reaction.account_id,
                        "whatsapp_web_reaction",
                        reaction.provider_record_id(),
                        reaction.source_fingerprint(),
                        &reaction.import_batch_id,
                        json!({"reaction": reaction.reaction}),
                    ),
                })
            })
        }

        fn reconcile_fixture_reaction_commands<'a>(
            &'a self,
            _reaction: &'a NewWhatsappWebReaction,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_reaction_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_media<'a>(
            &'a self,
            media: &'a NewWhatsappWebMedia,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMedia> {
            Box::pin(async move {
                self.record_call("ingest_fixture_media");
                Ok(WhatsappWebObservedMedia {
                    raw: NewRawCommunicationRecord::new(
                        "raw-media-fake",
                        &media.account_id,
                        "whatsapp_web_media",
                        media.provider_record_id(),
                        media.source_fingerprint(),
                        &media.import_batch_id,
                        json!({"provider_attachment_id": media.provider_attachment_id}),
                    ),
                })
            })
        }

        fn reconcile_fixture_media_commands<'a>(
            &'a self,
            _media: &'a NewWhatsappWebMedia,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_media_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_status<'a>(
            &'a self,
            status: &'a NewWhatsappWebStatus,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatus> {
            Box::pin(async move {
                self.record_call("ingest_fixture_status");
                Ok(WhatsappWebObservedStatus {
                    raw: NewRawCommunicationRecord::new(
                        "raw-status-fake",
                        &status.account_id,
                        "whatsapp_web_status",
                        &status.provider_status_id,
                        status.source_fingerprint(),
                        &status.import_batch_id,
                        json!({"text": status.text}),
                    ),
                })
            })
        }

        fn ingest_fixture_status_view<'a>(
            &'a self,
            status_view: &'a NewWhatsappWebStatusView,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusView> {
            Box::pin(async move {
                self.record_call("ingest_fixture_status_view");
                Ok(WhatsappWebObservedStatusView {
                    raw: NewRawCommunicationRecord::new(
                        "raw-status-view-fake",
                        &status_view.account_id,
                        "whatsapp_web_status_view",
                        status_view.provider_record_id(),
                        status_view.source_fingerprint(),
                        &status_view.import_batch_id,
                        json!({"viewer_id": status_view.viewer_id}),
                    ),
                })
            })
        }

        fn ingest_fixture_status_delete<'a>(
            &'a self,
            status_delete: &'a NewWhatsappWebStatusDelete,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusDelete> {
            Box::pin(async move {
                self.record_call("ingest_fixture_status_delete");
                Ok(WhatsappWebObservedStatusDelete {
                    raw: NewRawCommunicationRecord::new(
                        "raw-status-delete-fake",
                        &status_delete.account_id,
                        "whatsapp_web_status_delete",
                        &status_delete.provider_status_id,
                        status_delete.source_fingerprint(),
                        &status_delete.import_batch_id,
                        json!({"reason_class": status_delete.reason_class}),
                    ),
                })
            })
        }

        fn ingest_fixture_presence<'a>(
            &'a self,
            presence: &'a NewWhatsappWebPresence,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedPresence> {
            Box::pin(async move {
                self.record_call("ingest_fixture_presence");
                Ok(WhatsappWebObservedPresence {
                    raw: NewRawCommunicationRecord::new(
                        "raw-presence-fake",
                        &presence.account_id,
                        "whatsapp_web_presence",
                        presence.provider_record_id(),
                        presence.source_fingerprint(),
                        &presence.import_batch_id,
                        json!({"presence_state": presence.presence_state}),
                    ),
                })
            })
        }

        fn ingest_fixture_call<'a>(
            &'a self,
            call: &'a NewWhatsappWebCall,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedCall> {
            Box::pin(async move {
                self.record_call("ingest_fixture_call");
                Ok(WhatsappWebObservedCall {
                    raw: NewRawCommunicationRecord::new(
                        "raw-call-fake",
                        &call.account_id,
                        "whatsapp_web_call",
                        &call.provider_call_id,
                        call.source_fingerprint(),
                        &call.import_batch_id,
                        json!({"call_state": call.call_state}),
                    ),
                })
            })
        }

        fn ingest_fixture_runtime_event<'a>(
            &'a self,
            runtime_event: &'a NewWhatsappWebRuntimeEvent,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedRuntimeEvent> {
            Box::pin(async move {
                self.record_call("ingest_fixture_runtime_event");
                Ok(WhatsappWebObservedRuntimeEvent {
                    raw: NewRawCommunicationRecord::new(
                        "raw-runtime-event-fake",
                        &runtime_event.account_id,
                        "whatsapp_web_runtime_event",
                        &runtime_event.provider_event_id,
                        runtime_event.source_fingerprint(),
                        &runtime_event.import_batch_id,
                        json!({
                            "runtime_event_kind": runtime_event.runtime_event_kind,
                            "runtime_status": runtime_event.runtime_status,
                            "lifecycle_state": runtime_event.lifecycle_state,
                            "severity": runtime_event.severity,
                        }),
                    ),
                })
            })
        }

        fn reconcile_fixture_status_commands<'a>(
            &'a self,
            _status: &'a NewWhatsappWebStatus,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_status_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_dialog<'a>(
            &'a self,
            dialog: &'a NewWhatsappWebDialog,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedDialog> {
            Box::pin(async move {
                self.record_call("ingest_fixture_dialog");
                Ok(WhatsappWebObservedDialog {
                    raw: NewRawCommunicationRecord::new(
                        "raw-dialog-fake",
                        &dialog.account_id,
                        "whatsapp_web_dialog",
                        &dialog.provider_chat_id,
                        dialog.source_fingerprint(),
                        &dialog.import_batch_id,
                        json!({"chat_title": dialog.chat_title}),
                    ),
                })
            })
        }

        fn reconcile_fixture_dialog_commands<'a>(
            &'a self,
            _dialog: &'a NewWhatsappWebDialog,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_dialog_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_participant<'a>(
            &'a self,
            participant: &'a NewWhatsappWebParticipant,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedParticipant> {
            Box::pin(async move {
                self.record_call("ingest_fixture_participant");
                Ok(WhatsappWebObservedParticipant {
                    raw: NewRawCommunicationRecord::new(
                        "raw-participant-fake",
                        &participant.account_id,
                        "whatsapp_web_participant",
                        participant.provider_record_id(),
                        participant.source_fingerprint(),
                        &participant.import_batch_id,
                        json!({"display_name": participant.display_name}),
                    ),
                })
            })
        }

        fn reconcile_fixture_participant_commands<'a>(
            &'a self,
            _participant: &'a NewWhatsappWebParticipant,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_participant_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_message_update<'a>(
            &'a self,
            update: &'a NewWhatsappWebMessageUpdate,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageUpdate> {
            Box::pin(async move {
                self.record_call("ingest_fixture_message_update");
                Ok(WhatsappWebObservedMessageUpdate {
                    raw: NewRawCommunicationRecord::new(
                        "raw-message-update-fake",
                        &update.account_id,
                        "whatsapp_web_message_update",
                        &update.provider_message_id,
                        update.source_fingerprint(),
                        &update.import_batch_id,
                        json!({"text": update.text}),
                    ),
                })
            })
        }

        fn reconcile_fixture_message_update_commands<'a>(
            &'a self,
            _update: &'a NewWhatsappWebMessageUpdate,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_message_update_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_message_delete<'a>(
            &'a self,
            delete: &'a NewWhatsappWebMessageDelete,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageDelete> {
            Box::pin(async move {
                self.record_call("ingest_fixture_message_delete");
                Ok(WhatsappWebObservedMessageDelete {
                    raw: NewRawCommunicationRecord::new(
                        "raw-message-delete-fake",
                        &delete.account_id,
                        "whatsapp_web_message_delete",
                        &delete.provider_message_id,
                        delete.source_fingerprint(),
                        &delete.import_batch_id,
                        json!({"reason_class": delete.reason_class, "actor_class": delete.actor_class}),
                    ),
                })
            })
        }

        fn reconcile_fixture_message_delete_commands<'a>(
            &'a self,
            _delete: &'a NewWhatsappWebMessageDelete,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_message_delete_commands");
                Ok(Vec::new())
            })
        }

        fn ingest_fixture_receipt<'a>(
            &'a self,
            receipt: &'a NewWhatsappWebReceipt,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReceipt> {
            Box::pin(async move {
                self.record_call("ingest_fixture_receipt");
                Ok(WhatsappWebObservedReceipt {
                    raw: NewRawCommunicationRecord::new(
                        "raw-receipt-fake",
                        &receipt.account_id,
                        "whatsapp_web_receipt",
                        &receipt.provider_message_id,
                        receipt.source_fingerprint(),
                        &receipt.import_batch_id,
                        json!({"delivery_state": receipt.delivery_state.as_str()}),
                    ),
                })
            })
        }

        fn reconcile_fixture_receipt_commands<'a>(
            &'a self,
            _receipt: &'a NewWhatsappWebReceipt,
        ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
            Box::pin(async move {
                self.record_call("reconcile_fixture_receipt_commands");
                Ok(Vec::new())
            })
        }
    }

    fn fake_session(account_id: &str) -> WhatsappWebSession {
        let now = Utc::now();
        WhatsappWebSession {
            session_id: format!("session-{account_id}"),
            account_id: account_id.to_owned(),
            device_name: "Fake Runtime".to_owned(),
            companion_runtime: "fake".to_owned(),
            link_state: WhatsappWebLinkState::Linked.as_str().to_owned(),
            local_state_path: format!("docker/data/whatsapp/{account_id}"),
            last_sync_at: None,
            metadata: json!({"runtime": "fake"}),
            created_at: now,
            updated_at: now,
        }
    }

    fn fake_runtime_status(account_id: &str) -> WhatsAppRuntimeStatus {
        WhatsAppRuntimeStatus {
            account_id: account_id.to_owned(),
            provider_kind: "whatsapp_web".to_owned(),
            provider_shape: "whatsapp_business_cloud".to_owned(),
            runtime_kind: "fake_business_cloud".to_owned(),
            status: "stopped".to_owned(),
            fixture_runtime: false,
            live_runtime_available: true,
            live_send_available: false,
            qr_pairing_available: false,
            pair_code_available: false,
            media_download_available: false,
            media_upload_available: false,
            session_restore_available: false,
            session_secret_ref: None,
            runtime_blockers: vec![],
            last_error: None,
            updated_at: Utc::now(),
        }
    }

    fn fake_command_response(
        command_id: &str,
        idempotency_key: &str,
        command_kind: &str,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: Option<&str>,
    ) -> WhatsAppProviderCommandResponse {
        WhatsAppProviderCommandResponse {
            command_id: command_id.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            command_kind: command_kind.to_owned(),
            account_id: account_id.to_owned(),
            provider_kind: "whatsapp_web".to_owned(),
            provider_shape: "whatsapp_business_cloud".to_owned(),
            runtime_kind: "fake_business_cloud".to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.map(str::to_owned),
            status: "blocked".to_owned(),
            durable_status: "cancelled".to_owned(),
            delivery_state: "not_attempted".to_owned(),
            session_restore_available: false,
            rendered_preview_hash: None,
            runtime_blockers: vec!["fake_command_blocked".to_owned()],
            last_error: Some("fake_command_blocked".to_owned()),
            updated_at: Utc::now(),
        }
    }

    fn fake_provider_command(
        command_id: &str,
        idempotency_key: &str,
        command_kind: &str,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: Option<&str>,
        status: &str,
    ) -> WhatsAppProviderCommand {
        let now = Utc::now();
        WhatsAppProviderCommand {
            command_id: command_id.to_owned(),
            account_id: account_id.to_owned(),
            command_kind: command_kind.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.map(str::to_owned),
            capability_state: "blocked".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "confirmed".to_owned(),
            status: status.to_owned(),
            retry_count: 0,
            max_retries: 3,
            last_error: None,
            result_payload: json!({"status": status}),
            audit_metadata: json!({"rendered_preview_hash": "sha256:fake"}),
            provider_state: json!({}),
            reconciliation_status: "not_observed".to_owned(),
            next_attempt_at: None,
            last_attempt_at: None,
            provider_observed_at: None,
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn whatsapp_provider_runtime_application_service_uses_trait_boundary() {
        let runtime = Arc::new(FakeWhatsAppProviderRuntime::default());
        let service = WhatsappProviderRuntimeApplicationService::new(runtime.clone());
        assert_eq!(
            service.provider_shape(),
            WhatsAppProviderRuntimeShape::BusinessCloud
        );

        let setup_response = service
            .setup_fixture_account(&WhatsappWebAccountSetupRequest {
                account_id: "account-1".to_owned(),
                provider_kind: CommunicationProviderKind::WhatsappWeb,
                provider_shape: None,
                display_name: "WhatsApp runtime account".to_owned(),
                external_account_id: "external-1".to_owned(),
                device_name: "Hermes Desktop".to_owned(),
                local_state_path: "docker/data/whatsapp/account-1".to_owned(),
            })
            .await
            .expect("setup through fake runtime");
        assert_eq!(setup_response.runtime, "fake_business_cloud");

        let sessions = service
            .list_sessions(Some("account-1"), 10)
            .await
            .expect("list sessions through fake runtime");
        assert_eq!(sessions[0].account_id, "account-1");

        let messages = service
            .recent_messages(Some("account-1"), Some("chat-1"), 10)
            .await
            .expect("recent messages through fake runtime");
        assert_eq!(messages[0].provider_chat_id.as_deref(), Some("chat-1"));

        assert_eq!(
            runtime.calls(),
            vec!["setup_fixture_account", "list_sessions", "recent_messages"]
        );
    }
}
