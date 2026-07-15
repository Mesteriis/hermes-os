use super::*;

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
