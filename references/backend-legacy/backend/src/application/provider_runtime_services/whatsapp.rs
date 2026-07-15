use super::*;

impl WhatsappProviderRuntimeApplicationService {
    pub(crate) fn new(runtime: WhatsAppProviderRuntimeRef) -> Self {
        Self { runtime }
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
