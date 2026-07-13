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
    TelegramRuntimeStatus,
};
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebMessage, WhatsappWebSession,
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
    crate::integrations::whatsapp::runtime::whatsapp_web_companion_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
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
