use hermes_communications_api::accounts::ProviderAccountMutationOrigin;
use hermes_communications_api::accounts::{
    CommunicationProviderKind, ProviderAccount, ProviderSecretBindingCommandPort,
};
use hermes_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose,
};
mod business_cloud;
pub(crate) mod contracts;
mod native_md;
mod web_companion;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use qrcode::QrCode;
use qrcode::render::svg;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebError, WhatsappWebMessage, WhatsappWebObservedCall,
    WhatsappWebObservedDialog, WhatsappWebObservedMedia, WhatsappWebObservedMessage,
    WhatsappWebObservedMessageDelete, WhatsappWebObservedMessageUpdate,
    WhatsappWebObservedParticipant, WhatsappWebObservedPresence, WhatsappWebObservedReaction,
    WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent, WhatsappWebObservedStatus,
    WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView, WhatsappWebSession,
    WhatsappWebStore,
};
use crate::platform::communications::ProviderChannelMessageLookupPort;
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretResolver, SecretStoreKind,
};
use crate::vault::{HostVault, SecretEntryContext};
use contracts::*;

pub const WHATSAPP_OUTBOX_WORKER_ID: &str = "whatsapp-outbox-worker";
const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WhatsAppProviderWriteCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) idempotency_key: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) capability_state: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_decision: String,
    pub(crate) status: String,
    pub(crate) retry_count: i32,
    pub(crate) max_retries: i32,
    pub(crate) last_error: Option<String>,
    pub(crate) payload: Value,
    pub(crate) target_ref: Value,
    pub(crate) result_payload: Value,
    pub(crate) audit_metadata: Value,
    pub(crate) provider_state: Value,
    pub(crate) reconciliation_status: String,
    pub(crate) next_attempt_at: Option<DateTime<Utc>>,
    pub(crate) last_attempt_at: Option<DateTime<Utc>>,
    pub(crate) provider_observed_at: Option<DateTime<Utc>>,
    pub(crate) reconciled_at: Option<DateTime<Utc>>,
    pub(crate) dead_lettered_at: Option<DateTime<Utc>>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

struct ProviderCommandInsert<'a> {
    command_id: String,
    account_id: &'a str,
    command_kind: &'a str,
    idempotency_key: String,
    provider_chat_id: &'a str,
    provider_message_id: Option<&'a str>,
    action_class: &'a str,
    confirmation_decision: &'a str,
    payload: Value,
    target_ref: Value,
    rendered_preview_hash: Option<String>,
    restored_session_secret_ref: Option<String>,
}

pub fn whatsapp_web_companion_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    web_companion::build_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
    )
}

struct ShapedWhatsAppProviderRuntime {
    provider_shape: WhatsAppProviderRuntimeShape,
    inner: Arc<dyn WhatsAppProviderRuntime>,
    native_md_manager: Option<native_md::NativeMdRuntimeManager>,
    business_cloud_manager: Option<business_cloud::BusinessCloudRuntimeManager>,
}

impl ShapedWhatsAppProviderRuntime {
    fn new(
        provider_shape: WhatsAppProviderRuntimeShape,
        inner: Arc<dyn WhatsAppProviderRuntime>,
    ) -> Self {
        Self {
            provider_shape,
            inner,
            native_md_manager: None,
            business_cloud_manager: None,
        }
    }

    fn with_native_md_manager(mut self, manager: native_md::NativeMdRuntimeManager) -> Self {
        self.native_md_manager = Some(manager);
        self
    }

    fn with_business_cloud_manager(
        mut self,
        manager: business_cloud::BusinessCloudRuntimeManager,
    ) -> Self {
        self.business_cloud_manager = Some(manager);
        self
    }
}

macro_rules! delegate_inner_secret_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            secret_store: &'a SecretReferenceStore,
            vault: &'a HostVault,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(secret_store, vault, request)
        }
    };
}

macro_rules! delegate_inner_accountless_secret_method {
    ($method:ident, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            secret_store: &'a SecretReferenceStore,
            vault: &'a HostVault,
            account_id: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(secret_store, vault, account_id)
        }
    };
}

macro_rules! delegate_inner_request_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(request)
        }
    };
}

macro_rules! delegate_inner_fixture_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(request)
        }
    };
}

impl WhatsAppProviderRuntime for ShapedWhatsAppProviderRuntime {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        self.provider_shape
    }

    delegate_inner_accountless_secret_method!(runtime_status, WhatsAppRuntimeStatus);

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
                    .start_runtime(self.inner.as_ref(), secret_store, vault, request)
                    .await
            });
        }
        self.inner.start_runtime(secret_store, vault, request)
    }

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner.stop_runtime(secret_store, vault, request).await
            });
        }
        self.inner.stop_runtime(secret_store, vault, request)
    }

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .revoke_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.revoke_runtime(secret_store, vault, request)
    }

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .relink_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.relink_runtime(secret_store, vault, request)
    }

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .remove_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.remove_runtime(secret_store, vault, request)
    }

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let mut health = self
                    .inner
                    .runtime_health(secret_store, vault, account_id)
                    .await?;
                manager
                    .decorate_runtime_health(&mut health, account_id)
                    .await;
                Ok(health)
            });
        }
        if let Some(manager) = self.business_cloud_manager.as_ref() {
            return Box::pin(async move {
                let mut health = self
                    .inner
                    .runtime_health(secret_store, vault, account_id)
                    .await?;
                manager
                    .decorate_runtime_health(&mut health, account_id)
                    .await;
                Ok(health)
            });
        }
        self.inner.runtime_health(secret_store, vault, account_id)
    }
    fn start_qr_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppQrLinkStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppQrLinkSession> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
                    .start_qr_link(self.inner.as_ref(), secret_store, vault, request)
                    .await
            });
        }
        self.inner.start_qr_link(secret_store, vault, request)
    }

    fn start_pair_code_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppPairCodeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppPairCodeSession> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
                    .start_pair_code_link(self.inner.as_ref(), secret_store, vault, request)
                    .await
            });
        }
        self.inner
            .start_pair_code_link(secret_store, vault, request)
    }
    delegate_inner_secret_method!(
        request_send_text,
        WhatsAppTextSendRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_reply,
        WhatsAppReplyRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_forward,
        WhatsAppForwardRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_edit,
        WhatsAppEditRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_delete,
        WhatsAppDeleteRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_react,
        WhatsAppReactionRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_unreact,
        WhatsAppReactionRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_media_upload,
        WhatsAppMediaUploadRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_media_download,
        WhatsAppMediaDownloadRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_mark_read,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_mark_unread,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_archive,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_unarchive,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_mute,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_unmute,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_pin,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_unpin,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_join_group,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_leave_group,
        WhatsAppConversationCommandRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_publish_status,
        WhatsAppStatusPublishRequest,
        WhatsAppProviderCommandResponse
    );
    delegate_inner_secret_method!(
        request_send_voice_note,
        WhatsAppVoiceNoteSendRequest,
        WhatsAppProviderCommandResponse
    );

    fn execute_live_provider_command<'a>(
        &'a self,
        command: &'a WhatsAppProviderExecutableCommand,
    ) -> WhatsAppProviderCommandExecutionFuture<'a> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move { manager.execute_live_provider_command(command).await });
        }
        if let Some(manager) = self.business_cloud_manager.as_ref() {
            return Box::pin(async move { manager.execute_live_provider_command(command).await });
        }
        self.inner.execute_live_provider_command(command)
    }

    fn list_provider_commands<'a>(
        &'a self,
        account_id: &'a str,
        provider_chat_id: Option<&'a str>,
        provider_message_id: Option<&'a str>,
        command_kinds: &'a [String],
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse> {
        self.inner.list_provider_commands(
            account_id,
            provider_chat_id,
            provider_message_id,
            command_kinds,
            limit,
        )
    }

    fn manual_retry_provider_command<'a>(
        &'a self,
        command_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        self.inner.manual_retry_provider_command(command_id)
    }

    fn dead_letter_provider_command<'a>(
        &'a self,
        command_id: &'a str,
        reason: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        self.inner.dead_letter_provider_command(command_id, reason)
    }

    fn store_authorized_session_credential<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding> {
        self.inner
            .store_authorized_session_credential(secret_store, vault, credential)
    }

    delegate_inner_request_method!(
        setup_fixture_account,
        WhatsappWebAccountSetupRequest,
        WhatsappWebAccountSetupResponse
    );
    delegate_inner_request_method!(
        setup_live_blocked_account,
        WhatsappLiveAccountSetupRequest,
        WhatsappWebAccountSetupResponse
    );

    fn list_sessions<'a>(
        &'a self,
        account_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>> {
        self.inner.list_sessions(account_id, limit)
    }

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        provider_chat_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>> {
        self.inner
            .recent_messages(account_id, provider_chat_id, limit)
    }

    delegate_inner_fixture_method!(
        ingest_fixture_message,
        NewWhatsappWebMessage,
        WhatsappWebObservedMessage
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_message_commands,
        NewWhatsappWebMessage,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_reaction,
        NewWhatsappWebReaction,
        WhatsappWebObservedReaction
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_reaction_commands,
        NewWhatsappWebReaction,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_media,
        NewWhatsappWebMedia,
        WhatsappWebObservedMedia
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_media_commands,
        NewWhatsappWebMedia,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_status,
        NewWhatsappWebStatus,
        WhatsappWebObservedStatus
    );
    delegate_inner_fixture_method!(
        ingest_fixture_status_view,
        NewWhatsappWebStatusView,
        WhatsappWebObservedStatusView
    );
    delegate_inner_fixture_method!(
        ingest_fixture_status_delete,
        NewWhatsappWebStatusDelete,
        WhatsappWebObservedStatusDelete
    );
    delegate_inner_fixture_method!(
        ingest_fixture_presence,
        NewWhatsappWebPresence,
        WhatsappWebObservedPresence
    );
    delegate_inner_fixture_method!(
        ingest_fixture_call,
        NewWhatsappWebCall,
        WhatsappWebObservedCall
    );
    delegate_inner_fixture_method!(
        ingest_fixture_runtime_event,
        NewWhatsappWebRuntimeEvent,
        WhatsappWebObservedRuntimeEvent
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_status_commands,
        NewWhatsappWebStatus,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_dialog,
        NewWhatsappWebDialog,
        WhatsappWebObservedDialog
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_dialog_commands,
        NewWhatsappWebDialog,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_participant,
        NewWhatsappWebParticipant,
        WhatsappWebObservedParticipant
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_participant_commands,
        NewWhatsappWebParticipant,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_message_update,
        NewWhatsappWebMessageUpdate,
        WhatsappWebObservedMessageUpdate
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_message_update_commands,
        NewWhatsappWebMessageUpdate,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_message_delete,
        NewWhatsappWebMessageDelete,
        WhatsappWebObservedMessageDelete
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_message_delete_commands,
        NewWhatsappWebMessageDelete,
        Vec<WhatsAppProviderCommand>
    );
    delegate_inner_fixture_method!(
        ingest_fixture_receipt,
        NewWhatsappWebReceipt,
        WhatsappWebObservedReceipt
    );
    delegate_inner_fixture_method!(
        reconcile_fixture_receipt_commands,
        NewWhatsappWebReceipt,
        Vec<WhatsAppProviderCommand>
    );
}

pub fn whatsapp_native_md_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    native_md::build_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
        event_sink,
    )
}

pub fn whatsapp_business_cloud_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    business_cloud::build_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
    )
}

pub fn whatsapp_native_md_runtime_feature_enabled() -> bool {
    native_md::native_md_live_runtime_enabled()
}

pub fn whatsapp_business_cloud_runtime_feature_enabled() -> bool {
    business_cloud::business_cloud_live_runtime_enabled()
}

pub fn whatsapp_provider_runtime_mux(
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    web_companion_runtime: Arc<dyn WhatsAppProviderRuntime>,
    native_md_runtime: Arc<dyn WhatsAppProviderRuntime>,
    business_cloud_runtime: Arc<dyn WhatsAppProviderRuntime>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    Arc::new(WhatsAppProviderRuntimeMux {
        provider_account_store,
        web_companion_runtime,
        native_md_runtime,
        business_cloud_runtime,
    })
}

struct WhatsAppProviderRuntimeMux {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    web_companion_runtime: Arc<dyn WhatsAppProviderRuntime>,
    native_md_runtime: Arc<dyn WhatsAppProviderRuntime>,
    business_cloud_runtime: Arc<dyn WhatsAppProviderRuntime>,
}

impl WhatsAppProviderRuntimeMux {
    fn runtime_for_shape(
        &self,
        provider_shape: WhatsAppProviderRuntimeShape,
    ) -> Arc<dyn WhatsAppProviderRuntime> {
        match provider_shape {
            WhatsAppProviderRuntimeShape::WebCompanion => self.web_companion_runtime.clone(),
            WhatsAppProviderRuntimeShape::NativeMultiDevice => self.native_md_runtime.clone(),
            WhatsAppProviderRuntimeShape::BusinessCloud => self.business_cloud_runtime.clone(),
        }
    }

    fn all_runtimes(&self) -> [Arc<dyn WhatsAppProviderRuntime>; 3] {
        [
            self.web_companion_runtime.clone(),
            self.native_md_runtime.clone(),
            self.business_cloud_runtime.clone(),
        ]
    }

    async fn runtime_for_account_id(
        &self,
        account_id: &str,
    ) -> Result<Arc<dyn WhatsAppProviderRuntime>, WhatsappWebError> {
        let account = self
            .provider_account_store
            .get(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{account_id}` is not configured"
                ))
            })?;
        if !account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp provider account",
                account.account_id
            )));
        }
        Ok(self.runtime_for_shape(account_provider_shape(
            &account,
            WhatsAppProviderRuntimeShape::WebCompanion,
        )))
    }

    fn runtime_for_live_setup(
        &self,
        request: &WhatsappLiveAccountSetupRequest,
    ) -> Result<Arc<dyn WhatsAppProviderRuntime>, WhatsappWebError> {
        let provider_shape = match request.provider_shape.trim() {
            "whatsapp_native_md" => WhatsAppProviderRuntimeShape::NativeMultiDevice,
            "whatsapp_business_cloud" => WhatsAppProviderRuntimeShape::BusinessCloud,
            "whatsapp_web_companion" => WhatsAppProviderRuntimeShape::WebCompanion,
            other => {
                return Err(WhatsappWebError::InvalidRequest(format!(
                    "unsupported WhatsApp provider_shape `{other}`"
                )));
            }
        };
        Ok(self.runtime_for_shape(provider_shape))
    }
    async fn aggregate_sessions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebSession>, WhatsappWebError> {
        let mut sessions = Vec::new();
        for runtime in self.all_runtimes() {
            sessions.extend(runtime.list_sessions(account_id, limit).await?);
        }
        sessions.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        sessions.dedup_by(|left, right| left.session_id == right.session_id);
        sessions.truncate(limit.max(0) as usize);
        Ok(sessions)
    }

    async fn aggregate_recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebMessage>, WhatsappWebError> {
        let mut messages = Vec::new();
        for runtime in self.all_runtimes() {
            messages.extend(
                runtime
                    .recent_messages(account_id, provider_chat_id, limit)
                    .await?,
            );
        }
        messages.sort_by(|left, right| {
            right
                .occurred_at
                .cmp(&left.occurred_at)
                .then_with(|| right.projected_at.cmp(&left.projected_at))
                .then_with(|| left.message_id.cmp(&right.message_id))
        });
        messages.dedup_by(|left, right| left.message_id == right.message_id);
        messages.truncate(limit.max(0) as usize);
        Ok(messages)
    }

    async fn manual_retry_across_runtimes(
        &self,
        command_id: &str,
    ) -> Result<Option<WhatsAppProviderCommand>, WhatsappWebError> {
        for runtime in self.all_runtimes() {
            if let Some(command) = runtime.manual_retry_provider_command(command_id).await? {
                return Ok(Some(command));
            }
        }
        Ok(None)
    }

    async fn dead_letter_across_runtimes(
        &self,
        command_id: &str,
        reason: &str,
    ) -> Result<Option<WhatsAppProviderCommand>, WhatsappWebError> {
        for runtime in self.all_runtimes() {
            if let Some(command) = runtime
                .dead_letter_provider_command(command_id, reason)
                .await?
            {
                return Ok(Some(command));
            }
        }
        Ok(None)
    }
}

macro_rules! delegate_account_request_with_secret {
    ($method:ident, $request_ty:ty) => {
        fn $method<'a>(
            &'a self,
            secret_store: &'a SecretReferenceStore,
            vault: &'a HostVault,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
            Box::pin(async move {
                let runtime = self.runtime_for_account_id(&request.account_id).await?;
                runtime.$method(secret_store, vault, request).await
            })
        }
    };
}

macro_rules! delegate_account_fixture_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            Box::pin(async move {
                let runtime = self.runtime_for_account_id(&request.account_id).await?;
                runtime.$method(request).await
            })
        }
    };
}

impl WhatsAppProviderRuntime for WhatsAppProviderRuntimeMux {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        WhatsAppProviderRuntimeShape::WebCompanion
    }

    fn runtime_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(account_id).await?;
            runtime
                .runtime_status(secret_store, vault, account_id)
                .await
        })
    }

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.start_runtime(secret_store, vault, request).await
        })
    }

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.stop_runtime(secret_store, vault, request).await
        })
    }

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.revoke_runtime(secret_store, vault, request).await
        })
    }

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.relink_runtime(secret_store, vault, request).await
        })
    }

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.remove_runtime(secret_store, vault, request).await
        })
    }

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(account_id).await?;
            runtime
                .runtime_health(secret_store, vault, account_id)
                .await
        })
    }

    fn start_qr_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppQrLinkStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppQrLinkSession> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime.start_qr_link(secret_store, vault, request).await
        })
    }

    fn start_pair_code_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppPairCodeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppPairCodeSession> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&request.account_id).await?;
            runtime
                .start_pair_code_link(secret_store, vault, request)
                .await
        })
    }

    delegate_account_request_with_secret!(request_send_text, WhatsAppTextSendRequest);
    delegate_account_request_with_secret!(request_reply, WhatsAppReplyRequest);
    delegate_account_request_with_secret!(request_forward, WhatsAppForwardRequest);
    delegate_account_request_with_secret!(request_edit, WhatsAppEditRequest);
    delegate_account_request_with_secret!(request_delete, WhatsAppDeleteRequest);
    delegate_account_request_with_secret!(request_react, WhatsAppReactionRequest);
    delegate_account_request_with_secret!(request_unreact, WhatsAppReactionRequest);
    delegate_account_request_with_secret!(request_media_upload, WhatsAppMediaUploadRequest);
    delegate_account_request_with_secret!(request_media_download, WhatsAppMediaDownloadRequest);
    delegate_account_request_with_secret!(request_mark_read, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_mark_unread, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_archive, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_unarchive, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_mute, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_unmute, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_pin, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_unpin, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_join_group, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_leave_group, WhatsAppConversationCommandRequest);
    delegate_account_request_with_secret!(request_publish_status, WhatsAppStatusPublishRequest);
    delegate_account_request_with_secret!(request_send_voice_note, WhatsAppVoiceNoteSendRequest);

    fn execute_live_provider_command<'a>(
        &'a self,
        command: &'a WhatsAppProviderExecutableCommand,
    ) -> WhatsAppProviderCommandExecutionFuture<'a> {
        Box::pin(async move {
            let runtime = self
                .runtime_for_account_id(&command.account_id)
                .await
                .map_err(|error| {
                    WhatsAppProviderCommandExecutionError::new(
                        "whatsapp_runtime_account_lookup_failed",
                        error.to_string(),
                        Some(30),
                    )
                })?;
            runtime.execute_live_provider_command(command).await
        })
    }

    fn list_provider_commands<'a>(
        &'a self,
        account_id: &'a str,
        provider_chat_id: Option<&'a str>,
        provider_message_id: Option<&'a str>,
        command_kinds: &'a [String],
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(account_id).await?;
            runtime
                .list_provider_commands(
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    command_kinds,
                    limit,
                )
                .await
        })
    }

    fn manual_retry_provider_command<'a>(
        &'a self,
        command_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move { self.manual_retry_across_runtimes(command_id).await })
    }

    fn dead_letter_provider_command<'a>(
        &'a self,
        command_id: &'a str,
        reason: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move { self.dead_letter_across_runtimes(command_id, reason).await })
    }

    fn store_authorized_session_credential<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding> {
        Box::pin(async move {
            let runtime = self.runtime_for_account_id(&credential.account_id).await?;
            runtime
                .store_authorized_session_credential(secret_store, vault, credential)
                .await
        })
    }

    fn setup_fixture_account<'a>(
        &'a self,
        request: &'a WhatsappWebAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        self.web_companion_runtime.setup_fixture_account(request)
    }

    fn setup_live_blocked_account<'a>(
        &'a self,
        request: &'a WhatsappLiveAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        Box::pin(async move {
            let runtime = self.runtime_for_live_setup(request)?;
            runtime.setup_live_blocked_account(request).await
        })
    }

    fn list_sessions<'a>(
        &'a self,
        account_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>> {
        Box::pin(async move {
            match account_id {
                Some(account_id) => {
                    let runtime = self.runtime_for_account_id(account_id).await?;
                    runtime.list_sessions(Some(account_id), limit).await
                }
                None => self.aggregate_sessions(None, limit).await,
            }
        })
    }

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        provider_chat_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>> {
        Box::pin(async move {
            match account_id {
                Some(account_id) => {
                    let runtime = self.runtime_for_account_id(account_id).await?;
                    runtime
                        .recent_messages(Some(account_id), provider_chat_id, limit)
                        .await
                }
                None => {
                    self.aggregate_recent_messages(None, provider_chat_id, limit)
                        .await
                }
            }
        })
    }

    delegate_account_fixture_method!(
        ingest_fixture_message,
        NewWhatsappWebMessage,
        WhatsappWebObservedMessage
    );
    delegate_account_fixture_method!(
        reconcile_fixture_message_commands,
        NewWhatsappWebMessage,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_reaction,
        NewWhatsappWebReaction,
        WhatsappWebObservedReaction
    );
    delegate_account_fixture_method!(
        reconcile_fixture_reaction_commands,
        NewWhatsappWebReaction,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_media,
        NewWhatsappWebMedia,
        WhatsappWebObservedMedia
    );
    delegate_account_fixture_method!(
        reconcile_fixture_media_commands,
        NewWhatsappWebMedia,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_status,
        NewWhatsappWebStatus,
        WhatsappWebObservedStatus
    );
    delegate_account_fixture_method!(
        ingest_fixture_status_view,
        NewWhatsappWebStatusView,
        WhatsappWebObservedStatusView
    );
    delegate_account_fixture_method!(
        ingest_fixture_status_delete,
        NewWhatsappWebStatusDelete,
        WhatsappWebObservedStatusDelete
    );
    delegate_account_fixture_method!(
        ingest_fixture_presence,
        NewWhatsappWebPresence,
        WhatsappWebObservedPresence
    );
    delegate_account_fixture_method!(
        ingest_fixture_call,
        NewWhatsappWebCall,
        WhatsappWebObservedCall
    );
    delegate_account_fixture_method!(
        ingest_fixture_runtime_event,
        NewWhatsappWebRuntimeEvent,
        WhatsappWebObservedRuntimeEvent
    );
    delegate_account_fixture_method!(
        reconcile_fixture_status_commands,
        NewWhatsappWebStatus,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_dialog,
        NewWhatsappWebDialog,
        WhatsappWebObservedDialog
    );
    delegate_account_fixture_method!(
        reconcile_fixture_dialog_commands,
        NewWhatsappWebDialog,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_participant,
        NewWhatsappWebParticipant,
        WhatsappWebObservedParticipant
    );
    delegate_account_fixture_method!(
        reconcile_fixture_participant_commands,
        NewWhatsappWebParticipant,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_message_update,
        NewWhatsappWebMessageUpdate,
        WhatsappWebObservedMessageUpdate
    );
    delegate_account_fixture_method!(
        reconcile_fixture_message_update_commands,
        NewWhatsappWebMessageUpdate,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_message_delete,
        NewWhatsappWebMessageDelete,
        WhatsappWebObservedMessageDelete
    );
    delegate_account_fixture_method!(
        reconcile_fixture_message_delete_commands,
        NewWhatsappWebMessageDelete,
        Vec<WhatsAppProviderCommand>
    );
    delegate_account_fixture_method!(
        ingest_fixture_receipt,
        NewWhatsappWebReceipt,
        WhatsappWebObservedReceipt
    );
    delegate_account_fixture_method!(
        reconcile_fixture_receipt_commands,
        NewWhatsappWebReceipt,
        Vec<WhatsAppProviderCommand>
    );
}

impl WhatsAppProviderRuntime for WhatsappWebStore {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        WhatsAppProviderRuntimeShape::WebCompanion
    }

    fn runtime_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            Ok(self.status_from_account(&account, "stopped", restored_session, None))
        })
    }

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let runtime_kind = account_runtime_kind(&account);
            let provider_shape = account_provider_shape(&account, self.provider_shape());
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let status = if runtime_kind == "fixture" {
                "running"
            } else if runtime_kind == "live_blocked" {
                "blocked"
            } else {
                "stopped"
            };
            let last_error = (runtime_kind == "live_blocked").then(|| match provider_shape {
                WhatsAppProviderRuntimeShape::NativeMultiDevice => format!(
                    "native WhatsApp multi-device runtime is blocked: {}",
                    provider_shape_runtime_feature_blocker(provider_shape)
                ),
                WhatsAppProviderRuntimeShape::BusinessCloud => format!(
                    "WhatsApp Business Cloud runtime is blocked: {}",
                    provider_shape_runtime_feature_blocker(provider_shape)
                ),
                WhatsAppProviderRuntimeShape::WebCompanion => {
                    "live WhatsApp runtime is blocked until an explicit provider runtime is accepted"
                        .to_owned()
                }
            });
            Ok(self.status_from_account(&account, status, restored_session, last_error))
        })
    }

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            Ok(self.status_from_account(&account, "linked", restored_session, None))
        })
    }

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            self.clear_session_secret_material(secret_store, vault, &request.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "revoked",
                    "whatsapp.runtime.revoke",
                )
                .await?;
            Ok(self.status_from_account(
                &updated,
                "revoked",
                None,
                Some("WhatsApp session was revoked and must be relinked".to_owned()),
            ))
        })
    }

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            self.clear_session_secret_material(secret_store, vault, &request.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "created",
                    "whatsapp.runtime.relink",
                )
                .await?;
            Ok(self.status_from_account(&updated, "link_required", None, None))
        })
    }

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let binding_refs = self
                .clear_account_secret_material(secret_store, vault, &account.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &account.account_id,
                    "removed",
                    "whatsapp.runtime.remove",
                )
                .await?;
            Ok(WhatsAppRuntimeRemoveResponse {
                account_id: updated.account_id,
                provider_kind: updated.provider_kind.as_str().to_owned(),
                removed: true,
                unbound_secret_refs: binding_refs,
                removed_at: Utc::now(),
            })
        })
    }

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let status = self.status_from_account(&account, "stopped", restored_session, None);
            let health_status = runtime_health_status(&status);
            let healthy = health_status == "available";
            let requires_visible_webview = status.provider_shape == "whatsapp_web_companion"
                && status.runtime_kind != "fixture";
            let provider_shape = status.provider_shape.clone();
            let runtime_kind = status.runtime_kind.clone();
            let mut checks = json!({
                "session_metadata_available": true,
                "session_restore_available": status.session_restore_available,
                "session_secret_ref": status.session_secret_ref,
                "live_runtime_available": status.live_runtime_available,
                "qr_pairing_available": status.qr_pairing_available,
                "pair_code_available": status.pair_code_available,
                "runtime_blockers": status.runtime_blockers,
                "session": {
                    "metadata_available": true,
                    "restore_available": status.session_restore_available,
                    "linked": matches!(status.status.as_str(), "linked" | "available" | "syncing" | "degraded"),
                    "link_required": matches!(status.status.as_str(), "created" | "link_required"),
                    "revoked": status.status == "revoked",
                    "secret_ref_bound": status.session_secret_ref.is_some(),
                },
                "storage": {
                    "binding_store": "host_vault",
                    "binding_purpose": "whatsapp_web_session_key",
                    "secret_ref_bound": status.session_secret_ref.is_some(),
                },
                "runtime": {
                    "lifecycle_state": status.status,
                    "fixture_runtime": status.fixture_runtime,
                    "kind": status.runtime_kind,
                    "provider_shape": status.provider_shape,
                    "live_runtime_available": status.live_runtime_available,
                    "live_send_available": status.live_send_available,
                    "media_download_available": status.media_download_available,
                    "media_upload_available": status.media_upload_available,
                },
                "webview": {
                    "required": requires_visible_webview,
                    "visible_runtime_available": status.runtime_kind == "webview_companion"
                        && status.live_runtime_available,
                    "visible_runtime_required": status.runtime_blockers.iter().any(|blocker| blocker == "whatsapp_visible_runtime_required"),
                    "companion_runtime": status.runtime_kind == "webview_companion",
                },
                "validation": {
                    "status": health_status,
                    "healthy": healthy,
                    "blocker_count": status.runtime_blockers.len(),
                    "has_last_error": status.last_error.is_some(),
                },
            });
            if provider_shape == WhatsAppProviderRuntimeShape::NativeMultiDevice.as_str() {
                let native_md_driver = native_md::native_md_runtime_driver_health_check();
                checks["native_md_driver"] = native_md_driver.clone();
                checks["runtime"]["native_driver"] = native_md_driver;
            }
            if provider_shape == WhatsAppProviderRuntimeShape::WebCompanion.as_str() {
                let web_companion_bridge =
                    web_companion::web_companion_bridge_contract_health_check();
                checks["web_companion_bridge"] = web_companion_bridge.clone();
                checks["runtime"]["web_companion_bridge"] = web_companion_bridge;
            }
            Ok(WhatsAppRuntimeHealth {
                account_id: status.account_id,
                provider_shape,
                runtime_kind,
                status: health_status.to_owned(),
                healthy,
                checks,
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
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_shape = account_provider_shape(&account, self.provider_shape());
            ensure_qr_pairing_supported(provider_shape, "qr_link_start")?;
            let account = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "qr_pending",
                    "whatsapp.runtime.qr_link.start",
                )
                .await?;
            self.update_session_link_state(
                &account.account_id,
                "qr_pending",
                "whatsapp.runtime.qr_link.start",
            )
            .await?;
            let runtime_kind = account_runtime_kind(&account);
            let setup_id = setup_id("wa-qr", &request.account_id);
            let qr_svg = if runtime_kind == "fixture" {
                Some(render_fixture_whatsapp_qr_svg(
                    &fixture_whatsapp_qr_payload(&request.account_id, &setup_id),
                )?)
            } else {
                None
            };
            let expires_at = if runtime_kind == "fixture" {
                Some(Utc::now() + chrono::Duration::minutes(10))
            } else {
                None
            };
            Ok(WhatsAppQrLinkSession {
                account_id: account.account_id,
                provider_shape: provider_shape.as_str().to_owned(),
                runtime_kind: runtime_kind.clone(),
                status: "qr_pending".to_owned(),
                setup_id,
                qr_svg,
                expires_at,
                runtime_blockers: qr_pair_code_blockers(&runtime_kind, provider_shape),
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
            let phone_number = validate_non_empty("phone_number", &request.phone_number)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_shape = account_provider_shape(&account, self.provider_shape());
            ensure_qr_pairing_supported(provider_shape, "pair_code_link_start")?;
            let account = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "pair_code_pending",
                    "whatsapp.runtime.pair_code.start",
                )
                .await?;
            self.update_session_link_state(
                &account.account_id,
                "pair_code_pending",
                "whatsapp.runtime.pair_code.start",
            )
            .await?;
            let runtime_kind = account_runtime_kind(&account);
            let setup_id = setup_id("wa-pair", &request.account_id);
            let pair_code = if runtime_kind == "fixture" {
                Some(fixture_whatsapp_pair_code(
                    &request.account_id,
                    &phone_number,
                    &setup_id,
                ))
            } else {
                None
            };
            let expires_at = if runtime_kind == "fixture" {
                Some(Utc::now() + chrono::Duration::minutes(10))
            } else {
                None
            };
            Ok(WhatsAppPairCodeSession {
                account_id: account.account_id,
                provider_shape: provider_shape.as_str().to_owned(),
                runtime_kind: runtime_kind.clone(),
                status: "pair_code_pending".to_owned(),
                setup_id,
                phone_number,
                pair_code,
                expires_at,
                runtime_blockers: qr_pair_code_blockers(&runtime_kind, provider_shape),
            })
        })
    }

    fn request_send_text<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppTextSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_text",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_reply<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReplyRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let reply_to_provider_message_id = validate_non_empty(
                "reply_to_provider_message_id",
                &request.reply_to_provider_message_id,
            )?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "reply",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&reply_to_provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "text": text,
                        "reply_to_provider_message_id": reply_to_provider_message_id,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": reply_to_provider_message_id,
                    }),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_forward<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppForwardRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let _from_provider_chat_id =
                validate_non_empty("from_provider_chat_id", &request.from_provider_chat_id)?;
            let from_provider_message_id = validate_non_empty(
                "from_provider_message_id",
                &request.from_provider_message_id,
            )?;
            let text = request
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "forward",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&from_provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "from_provider_chat_id": _from_provider_chat_id,
                        "from_provider_message_id": from_provider_message_id,
                        "text": text,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": from_provider_message_id,
                    }),
                    rendered_preview_hash: text.map(whatsapp_text_preview_hash),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_edit<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppEditRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "edit",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_delete<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppDeleteRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let confirmation_decision = request
                .confirmation_decision
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("pending")
                .to_owned();
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "delete",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "destructive",
                    confirmation_decision: &confirmation_decision,
                    payload: json!({"delete_kind": "provider_delete"}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_react<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let reaction_emoji = validate_non_empty("reaction_emoji", &request.reaction_emoji)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "react",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"reaction_emoji": reaction_emoji}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unreact<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let reaction_emoji = validate_non_empty("reaction_emoji", &request.reaction_emoji)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unreact",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"reaction_emoji": reaction_emoji}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_media_upload<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaUploadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let blob_id = validate_non_empty("blob_id", &request.blob_id)?;
            let media_type = validate_non_empty("media_type", &request.media_type)?;
            let content_type = validate_non_empty("content_type", &request.content_type)?;
            let sha256 = validate_non_empty("sha256", &request.sha256)?;
            let scan_status = validate_non_empty("scan_status", &request.scan_status)?;
            if request.size_bytes < 0 {
                return Err(WhatsappWebError::InvalidRequest(
                    "size_bytes must not be negative".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_media",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "attachment_id": request.attachment_id,
                        "blob_id": blob_id,
                        "media_type": media_type,
                        "caption": request.caption,
                        "filename": request.filename,
                        "content_type": content_type,
                        "size_bytes": request.size_bytes,
                        "sha256": sha256,
                        "scan_status": scan_status,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "attachment_id": request.attachment_id,
                        "blob_id": request.blob_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_media_download<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaDownloadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            if request.provider_attachment_id.is_none() && request.provider_media_id.is_none() {
                return Err(WhatsappWebError::InvalidRequest(
                    "provider_attachment_id or provider_media_id is required".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let media_download_secret_ref =
                request.provider_media_id.as_deref().map(|fingerprint| {
                    whatsapp_native_md_media_download_secret_ref(&account.account_id, fingerprint)
                });
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "download_media",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "read",
                    confirmation_decision: "not_required",
                    payload: json!({
                        "provider_attachment_id": request.provider_attachment_id,
                        "provider_media_id": request.provider_media_id,
                        "media_download_secret_ref": media_download_secret_ref.clone(),
                        "filename": request.filename,
                        "content_type": request.content_type,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                        "provider_attachment_id": request.provider_attachment_id,
                        "provider_media_id": request.provider_media_id,
                        "media_download_secret_ref": media_download_secret_ref,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mark_read<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mark_read",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"read_state": "read"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mark_unread<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mark_unread",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"read_state": "unread"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_archive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "archive",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"archive_state": "archived"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unarchive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unarchive",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"archive_state": "main"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mute",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"mute_state": "muted"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unmute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unmute",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"mute_state": "unmuted"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_pin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "pin",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"pin_state": "pinned"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unpin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unpin",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"pin_state": "unpinned"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_join_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let invite_link = request
                .invite_link
                .as_deref()
                .map(|value| validate_non_empty("invite_link", value))
                .transpose()?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "join_group",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"invite_link": invite_link}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_leave_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let confirmation_decision = request
                .confirmation_decision
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("pending")
                .to_owned();
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "leave_group",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "destructive",
                    confirmation_decision: &confirmation_decision,
                    payload: json!({"membership_state": "left"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_publish_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppStatusPublishRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "publish_status",
                    idempotency_key,
                    provider_chat_id: "status-feed",
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({"provider_chat_id": "status-feed"}),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_send_voice_note<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppVoiceNoteSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let blob_id = validate_non_empty("blob_id", &request.blob_id)?;
            let content_type = validate_non_empty("content_type", &request.content_type)?;
            let sha256 = validate_non_empty("sha256", &request.sha256)?;
            let scan_status = validate_non_empty("scan_status", &request.scan_status)?;
            if request.size_bytes < 0 {
                return Err(WhatsappWebError::InvalidRequest(
                    "size_bytes must not be negative".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_voice_note",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "attachment_id": request.attachment_id,
                        "blob_id": blob_id,
                        "media_type": "voice_note",
                        "filename": request.filename,
                        "content_type": content_type,
                        "size_bytes": request.size_bytes,
                        "sha256": sha256,
                        "scan_status": scan_status,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "attachment_id": request.attachment_id,
                        "blob_id": request.blob_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn list_provider_commands<'a>(
        &'a self,
        account_id: &'a str,
        provider_chat_id: Option<&'a str>,
        provider_message_id: Option<&'a str>,
        command_kinds: &'a [String],
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let rows = sqlx::query(
                r#"
                SELECT *
                FROM whatsapp_provider_write_commands
                WHERE account_id = $1
                  AND ($2::text IS NULL OR provider_chat_id = $2)
                  AND ($3::text IS NULL OR provider_message_id = $3)
                  AND (cardinality($4::text[]) = 0 OR command_kind = ANY($4::text[]))
                ORDER BY created_at DESC
                LIMIT $5
                "#,
            )
            .bind(&account.account_id)
            .bind(provider_chat_id)
            .bind(provider_message_id)
            .bind(command_kinds)
            .bind(clamp_limit(limit))
            .fetch_all(self.pool())
            .await?;
            let commands = rows
                .into_iter()
                .map(row_to_whatsapp_provider_write_command)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(WhatsAppProviderCommand::from)
                .collect();
            Ok(WhatsAppProviderCommandListResponse { items: commands })
        })
    }

    fn manual_retry_provider_command<'a>(
        &'a self,
        command_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move {
            let command_id = validate_non_empty("command_id", command_id)?;
            let now = Utc::now();
            let row = sqlx::query(
                r#"
                UPDATE whatsapp_provider_write_commands
                SET status = 'retrying',
                    capability_state = CASE
                        WHEN command_kind IN (
                            'send_text', 'reply', 'forward',
                            'send_media', 'download_media', 'send_voice_note',
                            'edit', 'delete', 'react', 'unreact',
                            'mark_read', 'mark_unread',
                            'archive', 'unarchive',
                            'mute', 'unmute',
                            'pin', 'unpin',
                            'join_group', 'leave_group',
                            'publish_status'
                        ) THEN 'available'
                        ELSE capability_state
                    END,
                    retry_count = 0,
                    next_attempt_at = $2,
                    last_attempt_at = NULL,
                    locked_at = NULL,
                    locked_by = NULL,
                    provider_observed_at = NULL,
                    provider_state = '{}'::jsonb,
                    reconciliation_status = 'not_observed',
                    reconciled_at = NULL,
                    dead_lettered_at = NULL,
                    completed_at = NULL,
                    last_error = NULL,
                    result_payload = result_payload || jsonb_build_object('manual_retry_at', $2),
                    updated_at = $2
                WHERE command_id = $1
                  AND status IN ('failed', 'dead_letter', 'retrying', 'cancelled')
                RETURNING *
                "#,
            )
            .bind(command_id)
            .bind(now)
            .fetch_optional(self.pool())
            .await?;
            let command = row
                .map(row_to_whatsapp_provider_write_command)
                .transpose()?;
            if let Some(command) = command {
                self.mirror_canonical_provider_command(&command).await?;
                Ok(Some(WhatsAppProviderCommand::from(command)))
            } else {
                Ok(None)
            }
        })
    }

    fn dead_letter_provider_command<'a>(
        &'a self,
        command_id: &'a str,
        reason: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move {
            let command_id = validate_non_empty("command_id", command_id)?;
            let reason = validate_non_empty("reason", reason)?;
            let now = Utc::now();
            let row = sqlx::query(
                r#"
                UPDATE whatsapp_provider_write_commands
                SET status = 'dead_letter',
                    locked_at = NULL,
                    locked_by = NULL,
                    last_error = $3,
                    dead_lettered_at = $2,
                    updated_at = $2
                WHERE command_id = $1
                  AND status NOT IN ('completed', 'dead_letter')
                RETURNING *
                "#,
            )
            .bind(command_id)
            .bind(now)
            .bind(reason)
            .fetch_optional(self.pool())
            .await?;
            let command = row
                .map(row_to_whatsapp_provider_write_command)
                .transpose()?;
            if let Some(command) = command {
                self.mirror_canonical_provider_command(&command).await?;
                Ok(Some(WhatsAppProviderCommand::from(command)))
            } else {
                Ok(None)
            }
        })
    }

    fn store_authorized_session_credential<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding> {
        Box::pin(async move {
            let account = self.whatsapp_account(&credential.account_id).await?;
            let provider_shape = account_provider_shape(&account, self.provider_shape());
            ensure_session_secret_supported(provider_shape, "store_authorized_session_credential")?;
            let session_material =
                validate_non_empty("session_material", &credential.session_material)?;
            let label = validate_non_empty("label", &credential.label)?;
            let purpose = ProviderAccountSecretPurpose::WhatsappWebSessionKey;
            if !purpose.accepts_secret_kind(credential.secret_kind) {
                return Err(WhatsappWebError::InvalidRequest(format!(
                    "secret_kind `{}` is incompatible with {}",
                    credential.secret_kind.as_str(),
                    purpose.as_str()
                )));
            }

            let secret_ref = whatsapp_session_secret_ref(&account.account_id);
            let metadata = session_secret_metadata(&account, &credential.metadata);
            let runtime_kind = authorized_session_runtime_kind(&account, &credential.metadata);
            secret_store
                .upsert_secret_reference(
                    &NewSecretReference::new(
                        &secret_ref,
                        credential.secret_kind,
                        SecretStoreKind::HostVault,
                        format!("{label} for {}", account.account_id),
                    )
                    .metadata(metadata.clone()),
                )
                .await?;
            vault.store_secret(
                &secret_ref,
                &session_material,
                SecretEntryContext {
                    entry_kind: "provider_session",
                    account_id: &account.account_id,
                    purpose: purpose.as_str(),
                    secret_kind: credential.secret_kind.as_str(),
                    label: &label,
                    metadata: &metadata,
                },
            )?;
            self.provider_secret_binding_store()
                .bind(&NewProviderAccountSecretBinding::new(
                    &account.account_id,
                    purpose,
                    &secret_ref,
                ))
                .await
                .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
            if runtime_kind != account_runtime_kind(&account) {
                self.update_account_runtime_kind(
                    &account.account_id,
                    &runtime_kind,
                    "whatsapp.runtime.authorized_session.store",
                )
                .await?;
            }
            self.update_account_lifecycle_state(
                &account.account_id,
                "linked",
                "whatsapp.runtime.authorized_session.store",
            )
            .await?;
            self.update_session_link_state(
                &account.account_id,
                "linked",
                "whatsapp.runtime.authorized_session.store",
            )
            .await?;

            Ok(WhatsAppCredentialBinding {
                secret_purpose: purpose.as_str().to_owned(),
                secret_ref,
                secret_kind: credential.secret_kind,
                store_kind: SecretStoreKind::HostVault,
            })
        })
    }

    fn setup_fixture_account<'a>(
        &'a self,
        request: &'a WhatsappWebAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        Box::pin(async move { WhatsappWebStore::setup_fixture_account(self, request).await })
    }

    fn setup_live_blocked_account<'a>(
        &'a self,
        request: &'a WhatsappLiveAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        Box::pin(async move { WhatsappWebStore::setup_live_blocked_account(self, request).await })
    }

    fn list_sessions<'a>(
        &'a self,
        account_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>> {
        Box::pin(async move { WhatsappWebStore::list_sessions(self, account_id, limit).await })
    }

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        provider_chat_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>> {
        Box::pin(async move {
            WhatsappWebStore::recent_messages(self, account_id, provider_chat_id, limit).await
        })
    }

    fn ingest_fixture_message<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessage> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message(self, message).await })
    }

    fn reconcile_fixture_message_commands<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_commands(self, message).await
        })
    }

    fn ingest_fixture_reaction<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReaction> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_reaction(self, reaction).await })
    }

    fn reconcile_fixture_reaction_commands<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_reaction_commands(self, reaction).await
        })
    }

    fn ingest_fixture_media<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMedia> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_media(self, media).await })
    }

    fn reconcile_fixture_media_commands<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_media_commands(self, media).await },
        )
    }

    fn ingest_fixture_status<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatus> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_status(self, status).await })
    }

    fn ingest_fixture_status_view<'a>(
        &'a self,
        status_view: &'a NewWhatsappWebStatusView,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusView> {
        Box::pin(
            async move { WhatsappWebStore::ingest_fixture_status_view(self, status_view).await },
        )
    }

    fn ingest_fixture_status_delete<'a>(
        &'a self,
        status_delete: &'a NewWhatsappWebStatusDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusDelete> {
        Box::pin(async move {
            WhatsappWebStore::ingest_fixture_status_delete(self, status_delete).await
        })
    }

    fn ingest_fixture_presence<'a>(
        &'a self,
        presence: &'a NewWhatsappWebPresence,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedPresence> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_presence(self, presence).await })
    }

    fn ingest_fixture_call<'a>(
        &'a self,
        call: &'a NewWhatsappWebCall,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedCall> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_call(self, call).await })
    }

    fn ingest_fixture_runtime_event<'a>(
        &'a self,
        runtime_event: &'a NewWhatsappWebRuntimeEvent,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedRuntimeEvent> {
        Box::pin(async move {
            WhatsappWebStore::ingest_fixture_runtime_event(self, runtime_event).await
        })
    }

    fn reconcile_fixture_status_commands<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_status_commands(self, status).await },
        )
    }

    fn ingest_fixture_dialog<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedDialog> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_dialog(self, dialog).await })
    }

    fn reconcile_fixture_dialog_commands<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_dialog_commands(self, dialog).await },
        )
    }

    fn ingest_fixture_participant<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedParticipant> {
        Box::pin(
            async move { WhatsappWebStore::ingest_fixture_participant(self, participant).await },
        )
    }

    fn reconcile_fixture_participant_commands<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_participant_commands(self, participant).await
        })
    }

    fn ingest_fixture_message_update<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageUpdate> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message_update(self, update).await })
    }

    fn reconcile_fixture_message_update_commands<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_update_commands(self, update).await
        })
    }

    fn ingest_fixture_message_delete<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageDelete> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message_delete(self, delete).await })
    }

    fn reconcile_fixture_message_delete_commands<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_delete_commands(self, delete).await
        })
    }

    fn ingest_fixture_receipt<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReceipt> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_receipt(self, receipt).await })
    }

    fn reconcile_fixture_receipt_commands<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_receipt_commands(self, receipt).await
        })
    }
}

impl WhatsappWebStore {
    async fn update_account_runtime_kind(
        &self,
        account_id: &str,
        runtime_kind: &str,
        actor: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let mut config = account.config.clone();
        let Some(config_object) = config.as_object_mut() else {
            return Err(WhatsappWebError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        config_object.insert("runtime".to_owned(), json!(runtime_kind));
        self.provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                actor,
                "runtime_kind_update",
            )
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{}` is not configured",
                    account.account_id
                ))
            })
    }

    async fn update_account_lifecycle_state(
        &self,
        account_id: &str,
        lifecycle_state: &str,
        actor: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let mut config = account.config.clone();
        let Some(config_object) = config.as_object_mut() else {
            return Err(WhatsappWebError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        let now = Utc::now();
        config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
        config_object.insert("lifecycle_updated_at".to_owned(), json!(now));
        match lifecycle_state {
            "created" => {
                config_object.insert("created_at_runtime".to_owned(), json!(now));
            }
            "linked" => {
                config_object.insert("linked_at".to_owned(), json!(now));
            }
            "revoked" => {
                config_object.insert("revoked_at".to_owned(), json!(now));
            }
            "removed" => {
                config_object.insert("removed_at".to_owned(), json!(now));
            }
            _ => {}
        }

        self.provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                actor,
                lifecycle_state,
            )
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{}` is not configured",
                    account.account_id
                ))
            })
    }

    async fn reconcile_fixture_message_commands(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN ('send_text', 'reply', 'forward')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&message.account_id)
        .bind(&message.provider_chat_id)
        .bind(message.occurred_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let should_reconcile = match command.command_kind.as_str() {
                "send_text" => {
                    command.payload.get("text").and_then(Value::as_str)
                        == Some(message.text.as_str())
                }
                "reply" => {
                    command.payload.get("text").and_then(Value::as_str)
                        == Some(message.text.as_str())
                        && command
                            .payload
                            .get("reply_to_provider_message_id")
                            .and_then(Value::as_str)
                            == message.reply_to_provider_message_id.as_deref()
                }
                "forward" => {
                    command
                        .payload
                        .get("from_provider_chat_id")
                        .and_then(Value::as_str)
                        == message.forward_origin_chat_id.as_deref()
                        && command
                            .payload
                            .get("from_provider_message_id")
                            .and_then(Value::as_str)
                            == message.forward_origin_message_id.as_deref()
                }
                _ => false,
            };
            if !should_reconcile {
                continue;
            }

            let provider_state = json!({
                "provider_chat_id": message.provider_chat_id,
                "provider_message_id": message.provider_message_id,
                "delivery_state": message.delivery_state.as_str(),
                "reply_to_provider_message_id": message.reply_to_provider_message_id,
                "forward_origin_chat_id": message.forward_origin_chat_id,
                "forward_origin_message_id": message.forward_origin_message_id,
                "observed_via": "fixture_message",
            });
            let result_payload = json!({
                "provider_chat_id": message.provider_chat_id,
                "provider_message_id": message.provider_message_id,
                "delivery_state": message.delivery_state.as_str(),
                "provider_observed_at": message.occurred_at,
                "observed_via": "fixture_message",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    message.occurred_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_receipt_commands(
        &self,
        receipt: &NewWhatsappWebReceipt,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN (
                  'send_text', 'send_template', 'reply', 'forward', 'send_media',
                  'send_voice_note'
              )
              AND status IN ('queued', 'retrying', 'executing', 'completed')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND created_at <= $4
              AND (
                  provider_message_id = $3
                  OR provider_state #>> '{business_cloud,provider_request_id}' = $3
                  OR provider_state #>> '{business_cloud,provider_observed_completion_target,provider_message_id}' = $3
                  OR provider_state #>> '{native_md,provider_request_id}' = $3
                  OR provider_state #>> '{native_md,provider_observed_completion_target,provider_message_id}' = $3
                  OR result_payload #>> '{provider_submission,provider_request_id}' = $3
                  OR result_payload #>> '{provider_submission,provider_observed_completion_target,provider_message_id}' = $3
              )
            ORDER BY created_at ASC
            "#,
        )
        .bind(&receipt.account_id)
        .bind(&receipt.provider_chat_id)
        .bind(&receipt.provider_message_id)
        .bind(receipt.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            if !provider_request_id_matches_observed_receipt(&command, receipt) {
                continue;
            }
            let provider_state = json!({
                "provider_chat_id": receipt.provider_chat_id,
                "provider_message_id": receipt.provider_message_id,
                "delivery_state": receipt.delivery_state.as_str(),
                "provider_observed_at": receipt.observed_at,
                "observed_via": "fixture_receipt",
            });
            let result_payload = json!({
                "provider_chat_id": receipt.provider_chat_id,
                "provider_message_id": receipt.provider_message_id,
                "delivery_state": receipt.delivery_state.as_str(),
                "provider_observed_at": receipt.observed_at,
                "observed_via": "fixture_receipt",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    receipt.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_reaction_commands(
        &self,
        reaction: &NewWhatsappWebReaction,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind IN ('react', 'unreact')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&reaction.account_id)
        .bind(&reaction.provider_chat_id)
        .bind(&reaction.provider_message_id)
        .bind(reaction.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_reaction) = command
                .payload
                .get("reaction_emoji")
                .or_else(|| command.payload.get("reaction"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            if expected_reaction != reaction.reaction {
                continue;
            }

            let expected_active = match command.command_kind.as_str() {
                "react" => true,
                "unreact" => false,
                _ => continue,
            };

            let provider_state = json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
                "observed_via": "fixture_reaction",
            });
            let result_payload = json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
                "provider_observed_at": reaction.observed_at,
                "observed_via": "fixture_reaction",
            });

            let updated = if expected_active == reaction.is_active {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    reaction.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    reaction.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp reaction state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_media_commands(
        &self,
        media: &NewWhatsappWebMedia,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN ('send_media', 'send_voice_note', 'download_media')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&media.account_id)
        .bind(&media.provider_chat_id)
        .bind(media.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let should_reconcile = match command.command_kind.as_str() {
                "download_media" => {
                    command.provider_message_id.as_deref()
                        == Some(media.provider_message_id.as_str())
                        && command
                            .payload
                            .get("provider_attachment_id")
                            .and_then(Value::as_str)
                            .is_none_or(|value| value == media.provider_attachment_id)
                }
                "send_media" | "send_voice_note" => {
                    media.provider_message_id == format!("provider-message:{}", command.command_id)
                        || command
                            .payload
                            .get("blob_id")
                            .and_then(Value::as_str)
                            .is_some_and(|value| value == media.storage_path)
                        || provider_request_id_matches_observed_media(&command, media)
                }
                _ => false,
            };
            if !should_reconcile {
                continue;
            }

            let provider_state = json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "storage_path": media.storage_path,
                "sha256": media.sha256,
                "observed_via": "fixture_media",
            });
            let result_payload = json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "storage_path": media.storage_path,
                "sha256": media.sha256,
                "provider_observed_at": media.observed_at,
                "observed_via": "fixture_media",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    media.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_dialog_commands(
        &self,
        dialog: &NewWhatsappWebDialog,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id IS NULL
              AND command_kind IN (
                  'archive', 'unarchive',
                  'pin', 'unpin',
                  'mute', 'unmute',
                  'mark_read', 'mark_unread'
              )
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&dialog.account_id)
        .bind(&dialog.provider_chat_id)
        .bind(dialog.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        let executor_command_id = dialog
            .import_batch_id
            .strip_prefix("whatsapp-command:")
            .and_then(|value| value.rsplit_once(':'))
            .map(|(_, command_id)| command_id);
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            if executor_command_id.is_some_and(|command_id| command_id != command.command_id) {
                continue;
            }
            let (observed_state, state_key) = match command.command_kind.as_str() {
                "archive" | "unarchive" => match dialog.is_archived {
                    Some(state) => (state, "is_archived"),
                    None => continue,
                },
                "pin" | "unpin" => match dialog.is_pinned {
                    Some(state) => (state, "is_pinned"),
                    None => continue,
                },
                "mute" | "unmute" => match dialog.is_muted {
                    Some(state) => (state, "is_muted"),
                    None => continue,
                },
                "mark_read" | "mark_unread" => match dialog.is_unread {
                    Some(state) => (state, "is_unread"),
                    None => continue,
                },
                _ => continue,
            };
            let expected_state = match command.command_kind.as_str() {
                "archive" | "pin" | "mute" | "mark_unread" => true,
                "unarchive" | "unpin" | "unmute" | "mark_read" => false,
                _ => continue,
            };

            let provider_state = json!({
                "provider_chat_id": dialog.provider_chat_id,
                "chat_kind": dialog.chat_kind,
                "chat_title": dialog.chat_title,
                state_key: observed_state,
                "observed_via": "fixture_dialog",
            });
            let result_payload = json!({
                "provider_chat_id": dialog.provider_chat_id,
                "chat_kind": dialog.chat_kind,
                "chat_title": dialog.chat_title,
                state_key: observed_state,
                "provider_observed_at": dialog.observed_at,
                "observed_via": "fixture_dialog",
            });

            let updated = if observed_state == expected_state {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    dialog.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    dialog.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp dialog state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_participant_commands(
        &self,
        participant: &NewWhatsappWebParticipant,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id IS NULL
              AND command_kind IN ('join_group', 'leave_group')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required', 'pending')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&participant.account_id)
        .bind(&participant.provider_chat_id)
        .bind(participant.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let inferred_self_participant = participant.provider_member_id.trim().is_empty();
            if !participant.is_self && !inferred_self_participant {
                continue;
            }

            let observed_membership_matches = match command.command_kind.as_str() {
                "join_group" => matches!(participant.status.as_str(), "member" | "joined"),
                "leave_group" => participant.status == "left",
                _ => continue,
            };
            let provider_member_id = participant.effective_provider_member_id();
            let provider_state = json!({
                "provider_chat_id": participant.provider_chat_id,
                "provider_member_id": provider_member_id,
                "provider_identity_id": participant.provider_identity_id,
                "chat_kind": participant.effective_chat_kind(),
                "chat_title": participant.effective_chat_title(),
                "role": participant.role,
                "status": participant.status,
                "is_self": participant.is_self,
                "observed_via": "fixture_participant",
            });
            let result_payload = json!({
                "provider_chat_id": participant.provider_chat_id,
                "provider_member_id": provider_member_id,
                "provider_identity_id": participant.provider_identity_id,
                "chat_kind": participant.effective_chat_kind(),
                "chat_title": participant.effective_chat_title(),
                "role": participant.role,
                "status": participant.status,
                "is_self": participant.is_self,
                "provider_observed_at": participant.observed_at,
                "observed_via": "fixture_participant",
            });

            let updated = if observed_membership_matches {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    participant.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    participant.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp group membership state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_status_commands(
        &self,
        status: &NewWhatsappWebStatus,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = 'status-feed'
              AND command_kind = 'publish_status'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $2
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&status.account_id)
        .bind(status.occurred_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_text) = command.payload.get("text").and_then(Value::as_str) else {
                continue;
            };
            if expected_text != status.text {
                continue;
            }

            let provider_state = json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "text": status.text,
                "observed_via": "fixture_status",
            });
            let result_payload = json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "provider_observed_at": status.occurred_at,
                "observed_via": "fixture_status",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    status.occurred_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_message_update_commands(
        &self,
        update: &NewWhatsappWebMessageUpdate,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind = 'edit'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&update.account_id)
        .bind(&update.provider_chat_id)
        .bind(&update.provider_message_id)
        .bind(update.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_text) = command.payload.get("text").and_then(Value::as_str) else {
                continue;
            };
            let provider_state = json!({
                "provider_chat_id": update.provider_chat_id,
                "provider_message_id": update.provider_message_id,
                "text": update.text,
                "edited": true,
                "observed_via": "fixture_message_update",
            });
            let result_payload = json!({
                "provider_chat_id": update.provider_chat_id,
                "provider_message_id": update.provider_message_id,
                "text": update.text,
                "edited": true,
                "provider_observed_at": update.observed_at,
                "observed_via": "fixture_message_update",
            });
            let updated = if expected_text == update.text {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    update.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    update.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed different WhatsApp edited message content than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    async fn reconcile_fixture_message_delete_commands(
        &self,
        delete: &NewWhatsappWebMessageDelete,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind = 'delete'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&delete.account_id)
        .bind(&delete.provider_chat_id)
        .bind(&delete.provider_message_id)
        .bind(delete.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let provider_state = json!({
                "provider_chat_id": delete.provider_chat_id,
                "provider_message_id": delete.provider_message_id,
                "reason_class": delete.reason_class,
                "actor_class": delete.actor_class,
                "deleted": true,
                "observed_via": "fixture_message_delete",
            });
            let result_payload = json!({
                "provider_chat_id": delete.provider_chat_id,
                "provider_message_id": delete.provider_message_id,
                "reason_class": delete.reason_class,
                "actor_class": delete.actor_class,
                "deleted": true,
                "provider_observed_at": delete.observed_at,
                "observed_via": "fixture_message_delete",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    delete.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    async fn mark_provider_command_reconciled(
        &self,
        command_id: &str,
        observed_at: DateTime<Utc>,
        provider_state: Value,
        result_payload: Value,
    ) -> Result<WhatsAppProviderCommand, WhatsappWebError> {
        let resolved_provider_message_id =
            provider_message_id_from_state(&provider_state, &result_payload);
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                provider_message_id = COALESCE($5, provider_message_id),
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .bind(resolved_provider_message_id)
        .fetch_one(self.pool())
        .await?;
        let command = row_to_whatsapp_provider_write_command(row)?;
        self.mirror_canonical_provider_command(&command).await?;
        Ok(command.into())
    }

    async fn mark_provider_command_mismatch(
        &self,
        command_id: &str,
        observed_at: DateTime<Utc>,
        provider_state: Value,
        result_payload: Value,
        error_message: &str,
    ) -> Result<WhatsAppProviderCommand, WhatsappWebError> {
        let resolved_provider_message_id =
            provider_message_id_from_state(&provider_state, &result_payload);
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_provider_write_commands
            SET status = 'failed',
                result_payload = $3,
                last_error = $4,
                provider_observed_at = $2,
                provider_state = $5,
                provider_message_id = COALESCE($6, provider_message_id),
                reconciliation_status = 'mismatch',
                reconciled_at = $2,
                completed_at = NULL,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(error_message)
        .bind(&provider_state)
        .bind(resolved_provider_message_id)
        .fetch_one(self.pool())
        .await?;
        let command = row_to_whatsapp_provider_write_command(row)?;
        self.mirror_canonical_provider_command(&command).await?;
        Ok(command.into())
    }

    async fn whatsapp_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let account = self
            .provider_account_store()
            .get(&account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{account_id}` is not configured"
                ))
            })?;
        if !account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp provider account",
                account.account_id
            )));
        }
        Ok(account)
    }

    fn status_from_account(
        &self,
        account: &ProviderAccount,
        status: &str,
        restored_session_secret_ref: Option<String>,
        last_error: Option<String>,
    ) -> WhatsAppRuntimeStatus {
        let runtime_kind = account_runtime_kind(account);
        let provider_shape = account_provider_shape(account, self.provider_shape());
        let lifecycle_state = whatsapp_account_lifecycle_state(account);
        let forced_link_required = matches!(status, "link_required" | "created");
        let session_restore_available = restored_session_secret_ref.is_some()
            && !forced_link_required
            && !matches!(lifecycle_state, "revoked" | "removed");
        let session_secret_ref = if session_restore_available {
            restored_session_secret_ref
        } else {
            None
        };
        let effective_status = match status {
            "available" | "linked" | "revoked" | "removed" | "blocked" | "degraded" | "created"
            | "link_required" | "qr_pending" | "pair_code_pending" => status.to_owned(),
            _ if session_restore_available && runtime_kind == "fixture" && status == "running" => {
                "available".to_owned()
            }
            _ if lifecycle_state == "revoked" || lifecycle_state == "removed" => {
                lifecycle_state.to_owned()
            }
            _ if lifecycle_state == "qr_pending" || lifecycle_state == "pair_code_pending" => {
                lifecycle_state.to_owned()
            }
            _ if matches!(
                lifecycle_state,
                "linked" | "available" | "syncing" | "degraded" | "blocked"
            ) =>
            {
                lifecycle_state.to_owned()
            }
            _ if session_restore_available => "linked".to_owned(),
            _ => "link_required".to_owned(),
        };
        let live_runtime_available =
            runtime_runtime_available(&runtime_kind, provider_shape, &effective_status);
        let live_send_available =
            live_runtime_available && matches!(effective_status.as_str(), "available" | "degraded");
        let media_transfer_available =
            media_transfer_available(&runtime_kind, provider_shape, &effective_status);
        WhatsAppRuntimeStatus {
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: provider_shape.as_str().to_owned(),
            runtime_kind: runtime_kind.clone(),
            status: effective_status.clone(),
            fixture_runtime: runtime_kind == "fixture",
            live_runtime_available,
            live_send_available,
            qr_pairing_available: effective_status == "qr_pending",
            pair_code_available: effective_status == "pair_code_pending",
            media_download_available: media_transfer_available,
            media_upload_available: media_transfer_available,
            session_restore_available,
            session_secret_ref,
            runtime_blockers: runtime_status_blockers(
                &effective_status,
                provider_shape,
                &runtime_kind,
                session_restore_available,
                last_error.as_deref(),
            ),
            last_error,
            updated_at: Utc::now(),
        }
    }

    fn blocked_command_response(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderWriteCommand,
        rendered_preview_hash: Option<String>,
    ) -> WhatsAppProviderCommandResponse {
        let runtime_kind = account_runtime_kind(account);
        let provider_shape = account_provider_shape(account, self.provider_shape());
        let session_restore_available = command
            .audit_metadata
            .get("session_restore_available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        WhatsAppProviderCommandResponse {
            command_id: command.command_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            command_kind: command.command_kind.clone(),
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: provider_shape.as_str().to_owned(),
            runtime_kind,
            provider_chat_id: command.provider_chat_id.clone(),
            provider_message_id: command.provider_message_id.clone(),
            status: "blocked".to_owned(),
            durable_status: command.status.clone(),
            delivery_state: "not_attempted".to_owned(),
            session_restore_available,
            rendered_preview_hash,
            runtime_blockers: command
                .result_payload
                .get("runtime_blockers")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            last_error: command.last_error.clone(),
            updated_at: Utc::now(),
        }
    }

    async fn insert_blocked_provider_command(
        &self,
        input: ProviderCommandInsert<'_>,
    ) -> Result<WhatsAppProviderWriteCommand, WhatsappWebError> {
        let session_restore_available = input.restored_session_secret_ref.is_some();
        let account = self.whatsapp_account(input.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        let provider_shape = account_provider_shape(&account, self.provider_shape());
        ensure_provider_command_supported(provider_shape, input.command_kind)?;
        let blockers =
            provider_command_blockers(&runtime_kind, provider_shape, session_restore_available);
        let last_error = blockers.first().cloned();
        let result_payload = json!({
            "status": "blocked",
            "delivery_state": "not_attempted",
            "runtime_kind": runtime_kind,
            "runtime_blockers": blockers,
        });
        let audit_metadata = json!({
            "provider": "whatsapp",
            "provider_shape": provider_shape.as_str(),
            "runtime_kind": runtime_kind,
            "session_restore_available": session_restore_available,
            "rendered_preview_hash": input.rendered_preview_hash,
        });

        let mut transaction = self.pool().begin().await?;
        sqlx::query(
            r#"
            INSERT INTO whatsapp_provider_write_commands
                (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
                 provider_message_id, capability_state, action_class, confirmation_decision,
                 status, retry_count, max_retries, last_error, actor_id, payload, target_ref,
                 result_payload, audit_metadata, reconciliation_status)
            VALUES ($1, $2, $3, $4, $5, $6, 'blocked', $7, $8, 'cancelled', 0, 3, $9,
                    'hermes-frontend', $10, $11, $12, $13, 'not_required')
            ON CONFLICT (account_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(&input.command_id)
        .bind(input.account_id)
        .bind(input.command_kind)
        .bind(&input.idempotency_key)
        .bind(input.provider_chat_id)
        .bind(input.provider_message_id)
        .bind(input.action_class)
        .bind(input.confirmation_decision)
        .bind(last_error.as_deref())
        .bind(&input.payload)
        .bind(&input.target_ref)
        .bind(&result_payload)
        .bind(&audit_metadata)
        .execute(&mut *transaction)
        .await?;

        let command_row = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND idempotency_key = $2
            "#,
        )
        .bind(input.account_id)
        .bind(&input.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = command_row
            .map(row_to_whatsapp_provider_write_command)
            .transpose()?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp provider command `{}` was not persisted",
                    input.idempotency_key
                ))
            })?;

        sqlx::query(
            r#"
            INSERT INTO communication_accounts (
                account_id, provider_kind, display_name, external_account_id,
                config, metadata, created_at, updated_at
            )
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                jsonb_build_object('source_table', 'communication_provider_accounts'),
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&command.account_id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO communication_provider_commands (
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                provider_state, reconciliation_status, next_attempt_at, last_attempt_at,
                provider_observed_at, reconciled_at, dead_lettered_at, actor_id,
                happened_at, completed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, 'whatsapp', $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, 'hermes-frontend',
                $25, $26, $27, $28
            )
            ON CONFLICT (account_id, idempotency_key)
            DO UPDATE SET
                command_kind = EXCLUDED.command_kind,
                provider_conversation_id = EXCLUDED.provider_conversation_id,
                provider_message_id = EXCLUDED.provider_message_id,
                target_ref = EXCLUDED.target_ref,
                payload = EXCLUDED.payload,
                capability_state = EXCLUDED.capability_state,
                action_class = EXCLUDED.action_class,
                confirmation_decision = EXCLUDED.confirmation_decision,
                status = EXCLUDED.status,
                retry_count = EXCLUDED.retry_count,
                max_retries = EXCLUDED.max_retries,
                last_error = EXCLUDED.last_error,
                result_payload = EXCLUDED.result_payload,
                audit_metadata = EXCLUDED.audit_metadata,
                provider_state = EXCLUDED.provider_state,
                reconciliation_status = EXCLUDED.reconciliation_status,
                next_attempt_at = EXCLUDED.next_attempt_at,
                last_attempt_at = EXCLUDED.last_attempt_at,
                provider_observed_at = EXCLUDED.provider_observed_at,
                reconciled_at = EXCLUDED.reconciled_at,
                dead_lettered_at = EXCLUDED.dead_lettered_at,
                completed_at = EXCLUDED.completed_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&command.command_id)
        .bind(&command.account_id)
        .bind(&command.command_kind)
        .bind(&command.idempotency_key)
        .bind(&command.provider_chat_id)
        .bind(&command.provider_message_id)
        .bind(&command.target_ref)
        .bind(&command.payload)
        .bind(&command.capability_state)
        .bind(&command.action_class)
        .bind(&command.confirmation_decision)
        .bind(&command.status)
        .bind(command.retry_count)
        .bind(command.max_retries)
        .bind(&command.last_error)
        .bind(&command.result_payload)
        .bind(&command.audit_metadata)
        .bind(&command.provider_state)
        .bind(&command.reconciliation_status)
        .bind(command.next_attempt_at)
        .bind(command.last_attempt_at)
        .bind(command.provider_observed_at)
        .bind(command.reconciled_at)
        .bind(command.dead_lettered_at)
        .bind(command.created_at)
        .bind(command.completed_at)
        .bind(command.created_at)
        .bind(command.updated_at)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;

        self.provider_command_by_idempotency(input.account_id, &input.idempotency_key)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp provider command `{}` was not persisted",
                    input.idempotency_key
                ))
            })
    }

    async fn provider_command_by_idempotency(
        &self,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND idempotency_key = $2
            "#,
        )
        .bind(account_id)
        .bind(idempotency_key)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_whatsapp_provider_write_command).transpose()
    }

    async fn mirror_canonical_provider_command(
        &self,
        command: &WhatsAppProviderWriteCommand,
    ) -> Result<(), WhatsappWebError> {
        mirror_canonical_provider_command_for_pool(self.pool(), command).await
    }
}

async fn mirror_canonical_provider_command_for_pool(
    pool: &PgPool,
    command: &WhatsAppProviderWriteCommand,
) -> Result<(), WhatsappWebError> {
    sqlx::query(
        r#"
            INSERT INTO communication_accounts (
                account_id, provider_kind, display_name, external_account_id,
                config, metadata, created_at, updated_at
            )
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                jsonb_build_object('source_table', 'communication_provider_accounts'),
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = EXCLUDED.updated_at
            "#,
    )
    .bind(&command.account_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
            INSERT INTO communication_provider_commands (
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                provider_state, reconciliation_status, next_attempt_at, last_attempt_at,
                provider_observed_at, reconciled_at, dead_lettered_at, actor_id,
                happened_at, completed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, 'whatsapp', $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, 'hermes-frontend',
                $25, $26, $27, $28
            )
            ON CONFLICT (account_id, idempotency_key)
            DO UPDATE SET
                command_kind = EXCLUDED.command_kind,
                provider_conversation_id = EXCLUDED.provider_conversation_id,
                provider_message_id = EXCLUDED.provider_message_id,
                target_ref = EXCLUDED.target_ref,
                payload = EXCLUDED.payload,
                capability_state = EXCLUDED.capability_state,
                action_class = EXCLUDED.action_class,
                confirmation_decision = EXCLUDED.confirmation_decision,
                status = EXCLUDED.status,
                retry_count = EXCLUDED.retry_count,
                max_retries = EXCLUDED.max_retries,
                last_error = EXCLUDED.last_error,
                result_payload = EXCLUDED.result_payload,
                audit_metadata = EXCLUDED.audit_metadata,
                provider_state = EXCLUDED.provider_state,
                reconciliation_status = EXCLUDED.reconciliation_status,
                next_attempt_at = EXCLUDED.next_attempt_at,
                last_attempt_at = EXCLUDED.last_attempt_at,
                provider_observed_at = EXCLUDED.provider_observed_at,
                reconciled_at = EXCLUDED.reconciled_at,
                dead_lettered_at = EXCLUDED.dead_lettered_at,
                completed_at = EXCLUDED.completed_at,
                updated_at = EXCLUDED.updated_at
            "#,
    )
    .bind(&command.command_id)
    .bind(&command.account_id)
    .bind(&command.command_kind)
    .bind(&command.idempotency_key)
    .bind(&command.provider_chat_id)
    .bind(&command.provider_message_id)
    .bind(&command.target_ref)
    .bind(&command.payload)
    .bind(&command.capability_state)
    .bind(&command.action_class)
    .bind(&command.confirmation_decision)
    .bind(&command.status)
    .bind(command.retry_count)
    .bind(command.max_retries)
    .bind(&command.last_error)
    .bind(&command.result_payload)
    .bind(&command.audit_metadata)
    .bind(&command.provider_state)
    .bind(&command.reconciliation_status)
    .bind(command.next_attempt_at)
    .bind(command.last_attempt_at)
    .bind(command.provider_observed_at)
    .bind(command.reconciled_at)
    .bind(command.dead_lettered_at)
    .bind(command.created_at)
    .bind(command.completed_at)
    .bind(command.created_at)
    .bind(command.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

impl WhatsappWebStore {
    async fn optional_restored_session_secret_ref(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Option<String>, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let provider_shape = account_provider_shape(&account, self.provider_shape());
        let secret_purpose = provider_shape_restorable_secret_purpose(provider_shape);
        let binding = self
            .provider_secret_binding_store()
            .get_for_account(account_id, secret_purpose)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
        let Some(binding) = binding else {
            return Ok(None);
        };
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp session secret reference metadata not found: {}",
                    binding.secret_ref
                ))
            })?;
        if !binding
            .secret_purpose
            .accepts_secret_kind(reference.secret_kind)
        {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "secret_kind `{}` is incompatible with {}",
                reference.secret_kind.as_str(),
                binding.secret_purpose.as_str()
            )));
        }
        let resolved = vault.resolve(&reference).await?;
        if resolved.expose_for_runtime().trim().is_empty() {
            return Err(WhatsappWebError::InvalidRequest(
                "WhatsApp session material must not be empty".to_owned(),
            ));
        }
        Ok(Some(reference.secret_ref))
    }

    async fn clear_session_secret_material(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let provider_shape = account_provider_shape(&account, self.provider_shape());
        let secret_purpose = provider_shape_restorable_secret_purpose(provider_shape);
        self.clear_secret_material_for_bindings(
            secret_store,
            vault,
            vec![
                self.provider_secret_binding_store()
                    .get_for_account(account_id, secret_purpose)
                    .await
                    .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?,
            ],
            Some(secret_purpose),
            account_id,
        )
        .await
    }

    async fn clear_account_secret_material(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let bindings = self
            .provider_secret_binding_store()
            .list_for_account(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .into_iter()
            .map(Some)
            .collect();
        self.clear_secret_material_for_bindings(secret_store, vault, bindings, None, account_id)
            .await
    }

    async fn clear_secret_material_for_bindings(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        bindings: Vec<Option<hermes_communications_api::accounts::ProviderAccountSecretBinding>>,
        single_purpose: Option<ProviderAccountSecretPurpose>,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let mut removed_refs = Vec::new();
        for binding in bindings.into_iter().flatten() {
            let reference = secret_store.secret_reference(&binding.secret_ref).await?;
            self.provider_secret_binding_store()
                .unbind_for_account(account_id, binding.secret_purpose)
                .await
                .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
            secret_store
                .delete_secret_reference(&binding.secret_ref)
                .await?;
            if matches!(
                reference.as_ref().map(|entry| entry.store_kind),
                Some(SecretStoreKind::HostVault) | None
            ) {
                vault.delete_secret(&binding.secret_ref)?;
            }
            removed_refs.push(binding.secret_ref);
        }

        Ok(removed_refs)
    }
}

fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

fn account_provider_shape(
    account: &ProviderAccount,
    fallback: WhatsAppProviderRuntimeShape,
) -> WhatsAppProviderRuntimeShape {
    match account
        .config
        .get("provider_shape")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some("whatsapp_native_md") => WhatsAppProviderRuntimeShape::NativeMultiDevice,
        Some("whatsapp_business_cloud") => WhatsAppProviderRuntimeShape::BusinessCloud,
        Some("whatsapp_web_companion") => WhatsAppProviderRuntimeShape::WebCompanion,
        _ => match account.provider_kind {
            CommunicationProviderKind::WhatsappBusinessCloud => {
                WhatsAppProviderRuntimeShape::BusinessCloud
            }
            CommunicationProviderKind::WhatsappWeb => fallback,
            _ => fallback,
        },
    }
}

fn provider_shape_restorable_secret_purpose(
    provider_shape: WhatsAppProviderRuntimeShape,
) -> ProviderAccountSecretPurpose {
    match provider_shape {
        WhatsAppProviderRuntimeShape::BusinessCloud => {
            ProviderAccountSecretPurpose::WhatsappBusinessCloudAccessToken
        }
        WhatsAppProviderRuntimeShape::WebCompanion
        | WhatsAppProviderRuntimeShape::NativeMultiDevice => {
            ProviderAccountSecretPurpose::WhatsappWebSessionKey
        }
    }
}

fn row_to_whatsapp_provider_write_command(
    row: PgRow,
) -> Result<WhatsAppProviderWriteCommand, WhatsappWebError> {
    Ok(WhatsAppProviderWriteCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        last_error: row.try_get("last_error")?,
        payload: row.try_get("payload")?,
        target_ref: row.try_get("target_ref")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn provider_message_id_from_state<'a>(
    provider_state: &'a Value,
    result_payload: &'a Value,
) -> Option<&'a str> {
    result_payload
        .get("provider_message_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            provider_state
                .get("provider_message_id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        })
}

pub(crate) async fn claim_due_commands_for_execution(
    pool: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command.command_id
            FROM whatsapp_provider_write_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            WHERE command.status IN ('queued', 'retrying')
              AND command.retry_count < command.max_retries
              AND (command.next_attempt_at IS NULL OR command.next_attempt_at <= $1)
              AND command.confirmation_decision IN ('confirmed', 'not_required')
              AND command.capability_state IN ('available', 'degraded')
              AND account.provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
              AND COALESCE(account.config->>'runtime', '') = 'fixture'
              AND command.command_kind IN (
                  'send_text', 'reply', 'forward',
                  'send_media', 'download_media', 'send_voice_note',
                  'edit', 'delete', 'react', 'unreact',
                  'mark_read', 'mark_unread',
                  'archive', 'unarchive',
                  'mute', 'unmute',
                  'pin', 'unpin',
                  'join_group', 'leave_group',
                  'publish_status'
              )
            ORDER BY COALESCE(command.next_attempt_at, command.created_at) ASC,
                     command.created_at ASC,
                     command.command_id ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
        )
        UPDATE whatsapp_provider_write_commands command
        SET status = 'executing',
            retry_count = command.retry_count + 1,
            last_attempt_at = $1,
            locked_at = $1,
            locked_by = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            updated_at = $1
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .bind(WHATSAPP_OUTBOX_WORKER_ID)
    .fetch_all(pool)
    .await?;

    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

pub(crate) async fn claim_due_live_commands_for_execution(
    pool: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
    account_id: Option<&str>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command.command_id
            FROM whatsapp_provider_write_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            WHERE command.status IN ('queued', 'retrying')
              AND command.retry_count < command.max_retries
              AND (command.next_attempt_at IS NULL OR command.next_attempt_at <= $1)
              AND command.confirmation_decision IN ('confirmed', 'not_required')
              AND command.capability_state IN ('available', 'degraded')
              AND (
                    COALESCE(command.audit_metadata->>'session_restore_available', 'false') = 'true'
                 OR command.result_payload ? 'manual_retry_at'
              )
              AND account.provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
              AND COALESCE(account.config->>'runtime', '') NOT IN ('', 'fixture')
              AND (
                    COALESCE(account.config->>'lifecycle_state', '') IN (
                        'linked', 'available', 'syncing', 'degraded'
                    )
                 OR command.result_payload ? 'manual_retry_at'
              )
              AND ($4::text IS NULL OR command.account_id = $4)
              AND command.command_kind IN (
                  'send_text', 'reply', 'forward',
                  'send_media', 'download_media', 'send_voice_note',
                  'edit', 'delete', 'react', 'unreact',
                  'mark_read', 'mark_unread',
                  'archive', 'unarchive',
                  'mute', 'unmute',
                  'pin', 'unpin',
                  'join_group', 'leave_group',
                  'publish_status'
              )
            ORDER BY COALESCE(command.next_attempt_at, command.created_at) ASC,
                     command.created_at ASC,
                     command.command_id ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
        )
        UPDATE whatsapp_provider_write_commands command
        SET status = 'executing',
            retry_count = command.retry_count + 1,
            last_attempt_at = $1,
            locked_at = $1,
            locked_by = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            updated_at = $1
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .bind("whatsapp-runtime-bridge-worker")
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

pub(crate) async fn claim_due_native_md_commands_for_execution(
    pool: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command.command_id
            FROM whatsapp_provider_write_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            WHERE command.status IN ('queued', 'retrying')
              AND command.retry_count < command.max_retries
              AND (command.next_attempt_at IS NULL OR command.next_attempt_at <= $1)
              AND command.confirmation_decision IN ('confirmed', 'not_required')
              AND command.capability_state IN ('available', 'degraded')
              AND COALESCE(command.audit_metadata->>'session_restore_available', 'false') = 'true'
              AND account.provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
              AND COALESCE(account.config->>'provider_shape', '') = 'whatsapp_native_md'
              AND COALESCE(account.config->>'runtime', '') NOT IN ('', 'fixture', 'live_blocked')
              AND COALESCE(account.config->>'lifecycle_state', '') IN (
                  'linked', 'available', 'syncing', 'degraded'
              )
              AND command.command_kind IN (
                  'send_text', 'reply', 'forward',
                  'send_media', 'download_media', 'send_voice_note',
                  'edit', 'delete', 'react', 'unreact',
                  'mark_read', 'mark_unread',
                  'archive', 'unarchive',
                  'mute', 'unmute',
                  'pin', 'unpin',
                  'join_group', 'leave_group',
                  'publish_status'
              )
            ORDER BY COALESCE(command.next_attempt_at, command.created_at) ASC,
                     command.created_at ASC,
                     command.command_id ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
        )
        UPDATE whatsapp_provider_write_commands command
        SET status = 'executing',
            retry_count = command.retry_count + 1,
            last_attempt_at = $1,
            locked_at = $1,
            locked_by = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            updated_at = $1
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .bind("whatsapp-native-md-command-worker")
    .fetch_all(pool)
    .await?;

    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

pub(crate) async fn claim_due_business_cloud_commands_for_execution(
    pool: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command.command_id
            FROM whatsapp_provider_write_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            WHERE command.status IN ('queued', 'retrying')
              AND command.retry_count < command.max_retries
              AND (command.next_attempt_at IS NULL OR command.next_attempt_at <= $1)
              AND command.confirmation_decision IN ('confirmed', 'not_required')
              AND command.capability_state IN ('available', 'degraded')
              AND COALESCE(command.audit_metadata->>'session_restore_available', 'false') = 'true'
              AND account.provider_kind = 'whatsapp_business_cloud'
              AND COALESCE(account.config->>'provider_shape', '') = 'whatsapp_business_cloud'
              AND COALESCE(account.config->>'runtime', '') = 'business_cloud_smoke'
              AND COALESCE(account.config->>'business_cloud_live_smoke_enabled', 'false') = 'true'
              AND COALESCE(account.config->>'lifecycle_state', '') IN (
                  'linked', 'available', 'syncing', 'degraded'
              )
              AND command.command_kind IN ('send_text', 'send_template', 'send_media', 'send_voice_note')
            ORDER BY COALESCE(command.next_attempt_at, command.created_at) ASC,
                     command.created_at ASC,
                     command.command_id ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
        )
        UPDATE whatsapp_provider_write_commands command
        SET status = 'executing',
            retry_count = command.retry_count + 1,
            last_attempt_at = $1,
            locked_at = $1,
            locked_by = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            updated_at = $1
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .bind("whatsapp-business-cloud-command-worker")
    .fetch_all(pool)
    .await?;

    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

pub(crate) async fn import_canonical_provider_commands(
    pool: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let synced_rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT
                command.command_id,
                COALESCE(
                    NULLIF(trim(command.provider_conversation_id), ''),
                    NULLIF(trim(command.target_ref->>'provider_chat_id'), ''),
                    CASE
                        WHEN command.command_kind = 'publish_status' THEN 'status-feed'
                        ELSE NULL
                    END
                ) AS provider_chat_id,
                command.provider_message_id,
                command.capability_state,
                command.action_class,
                command.confirmation_decision,
                command.target_ref,
                command.payload,
                command.last_error,
                command.result_payload,
                command.audit_metadata || jsonb_build_object(
                    'imported_from_canonical_provider_command', true,
                    'canonical_imported_at', $1
                ) AS audit_metadata,
                CASE
                    WHEN command.status = 'confirmed' THEN 'queued'
                    ELSE command.status
                END AS status,
                command.retry_count,
                command.max_retries,
                command.completed_at,
                command.updated_at
            FROM communication_provider_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            JOIN whatsapp_provider_write_commands existing
              ON existing.command_id = command.command_id
            WHERE command.channel_kind = 'whatsapp'
              AND account.provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
              AND command.status IN ('queued', 'retrying', 'confirmed')
              AND command.command_kind IN (
                  'send_text', 'reply', 'forward',
                  'send_media', 'download_media', 'send_voice_note',
                  'edit', 'delete', 'react', 'unreact',
                  'mark_read', 'mark_unread',
                  'archive', 'unarchive',
                  'mute', 'unmute',
                  'pin', 'unpin',
                  'join_group', 'leave_group',
                  'publish_status'
              )
              AND existing.status IN ('queued', 'retrying', 'cancelled')
              AND (
                    existing.provider_chat_id IS DISTINCT FROM COALESCE(
                        NULLIF(trim(command.provider_conversation_id), ''),
                        NULLIF(trim(command.target_ref->>'provider_chat_id'), ''),
                        CASE
                            WHEN command.command_kind = 'publish_status' THEN 'status-feed'
                            ELSE NULL
                        END
                    )
                 OR existing.provider_message_id IS DISTINCT FROM command.provider_message_id
                 OR existing.capability_state IS DISTINCT FROM command.capability_state
                 OR existing.action_class IS DISTINCT FROM command.action_class
                 OR existing.confirmation_decision IS DISTINCT FROM command.confirmation_decision
                 OR existing.target_ref IS DISTINCT FROM command.target_ref
                 OR existing.payload IS DISTINCT FROM command.payload
                 OR existing.last_error IS DISTINCT FROM command.last_error
                 OR existing.result_payload IS DISTINCT FROM command.result_payload
                 OR existing.audit_metadata IS DISTINCT FROM (
                        command.audit_metadata || jsonb_build_object(
                            'imported_from_canonical_provider_command', true,
                            'canonical_imported_at', $1
                        )
                    )
                 OR existing.status IS DISTINCT FROM (
                        CASE
                            WHEN command.status = 'confirmed' THEN 'queued'
                            ELSE command.status
                        END
                    )
              )
            ORDER BY command.updated_at ASC, command.command_id ASC
            LIMIT $2
        )
        UPDATE whatsapp_provider_write_commands existing
        SET provider_chat_id = due.provider_chat_id,
            provider_message_id = due.provider_message_id,
            capability_state = due.capability_state,
            action_class = due.action_class,
            confirmation_decision = due.confirmation_decision,
            target_ref = due.target_ref,
            payload = due.payload,
            status = due.status,
            retry_count = CASE
                WHEN due.status IN ('queued', 'retrying') THEN due.retry_count
                ELSE existing.retry_count
            END,
            max_retries = due.max_retries,
            last_error = due.last_error,
            result_payload = due.result_payload,
            audit_metadata = due.audit_metadata,
            completed_at = due.completed_at,
            updated_at = GREATEST(existing.updated_at, due.updated_at, $1)
        FROM due
        WHERE existing.command_id = due.command_id
        RETURNING existing.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let synced_commands = synced_rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &synced_commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }

    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT
                command.command_id,
                command.account_id,
                command.command_kind,
                command.idempotency_key,
                COALESCE(
                    NULLIF(trim(command.provider_conversation_id), ''),
                    NULLIF(trim(command.target_ref->>'provider_chat_id'), ''),
                    CASE
                        WHEN command.command_kind = 'publish_status' THEN 'status-feed'
                        ELSE NULL
                    END
                ) AS provider_chat_id,
                command.provider_message_id,
                command.target_ref,
                command.payload,
                command.capability_state,
                command.action_class,
                command.confirmation_decision,
                CASE
                    WHEN command.status = 'confirmed' THEN 'queued'
                    ELSE command.status
                END AS status,
                command.retry_count,
                command.max_retries,
                command.last_error,
                command.result_payload,
                command.audit_metadata,
                command.actor_id,
                command.happened_at,
                command.completed_at,
                command.created_at,
                command.updated_at
            FROM communication_provider_commands command
            JOIN communication_provider_accounts account
              ON account.account_id = command.account_id
            LEFT JOIN whatsapp_provider_write_commands existing
              ON existing.command_id = command.command_id
            WHERE existing.command_id IS NULL
              AND command.channel_kind = 'whatsapp'
              AND account.provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
              AND command.status IN ('queued', 'retrying', 'confirmed')
              AND command.command_kind IN (
                  'send_text', 'reply', 'forward',
                  'send_media', 'download_media', 'send_voice_note',
                  'edit', 'delete', 'react', 'unreact',
                  'mark_read', 'mark_unread',
                  'archive', 'unarchive',
                  'mute', 'unmute',
                  'pin', 'unpin',
                  'join_group', 'leave_group',
                  'publish_status'
              )
              AND COALESCE(
                    NULLIF(trim(command.provider_conversation_id), ''),
                    NULLIF(trim(command.target_ref->>'provider_chat_id'), ''),
                    CASE
                        WHEN command.command_kind = 'publish_status' THEN 'status-feed'
                        ELSE NULL
                    END
                  ) IS NOT NULL
            ORDER BY command.created_at ASC, command.command_id ASC
            LIMIT $2
        )
        INSERT INTO whatsapp_provider_write_commands (
            command_id, account_id, command_kind, idempotency_key,
            provider_chat_id, provider_message_id, target_ref, payload,
            capability_state, action_class, confirmation_decision, status,
            retry_count, max_retries, last_error, result_payload, audit_metadata,
            actor_id, happened_at, completed_at, created_at, updated_at
        )
        SELECT
            due.command_id,
            due.account_id,
            due.command_kind,
            due.idempotency_key,
            due.provider_chat_id,
            due.provider_message_id,
            due.target_ref,
            due.payload,
            due.capability_state,
            due.action_class,
            due.confirmation_decision,
            due.status,
            due.retry_count,
            due.max_retries,
            due.last_error,
            due.result_payload,
            due.audit_metadata || jsonb_build_object(
                'imported_from_canonical_provider_command', true,
                'canonical_imported_at', $1
            ),
            COALESCE(NULLIF(trim(due.actor_id), ''), 'hermes-frontend'),
            due.happened_at,
            due.completed_at,
            due.created_at,
            GREATEST(due.updated_at, $1)
        FROM due
        RETURNING *
        "#,
    )
    .bind(now)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

async fn recover_stale_executing_commands_scoped(
    pool: &PgPool,
    now: DateTime<Utc>,
    fixture_runtime: Option<bool>,
    account_id: Option<&str>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let stale_before = now - chrono::Duration::seconds(STALE_EXECUTION_LOCK_SECONDS);
    let stale_rows = sqlx::query(
        r#"
        SELECT command.*
        FROM whatsapp_provider_write_commands command
        JOIN communication_provider_accounts account
          ON account.account_id = command.account_id
        WHERE status = 'executing'
          AND command.locked_at IS NOT NULL
          AND command.locked_at <= $1
          AND (
                $2::bool IS NULL
                OR ($2 = true AND COALESCE(account.config->>'runtime', '') = 'fixture')
                OR ($2 = false AND COALESCE(account.config->>'runtime', '') <> 'fixture')
              )
          AND ($3::text IS NULL OR command.account_id = $3)
        ORDER BY command.locked_at ASC, command.command_id ASC
        "#,
    )
    .bind(stale_before)
    .bind(fixture_runtime)
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    let stale_commands = stale_rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    let mut recovered = Vec::with_capacity(stale_commands.len());

    for command in stale_commands {
        let retry_after_seconds = if command.retry_count >= command.max_retries {
            None
        } else {
            Some(retry_delay_seconds(command.retry_count, None))
        };
        let failure_result_payload = json!({
            "failure": {
                "error_message": "WhatsApp provider command execution was interrupted before provider reconciliation",
                "error_code": "interrupted_execution",
                "retry_after_seconds": retry_after_seconds,
                "reported_at": now,
                "reported_via": "stale_execution_recovery",
            }
        });
        let failure_provider_state = json!({
            "last_failure": {
                "error_message": "WhatsApp provider command execution was interrupted before provider reconciliation",
                "error_code": "interrupted_execution",
                "retry_after_seconds": retry_after_seconds,
                "reported_at": now,
                "reported_via": "stale_execution_recovery",
            }
        });
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_provider_write_commands
            SET status = CASE
                    WHEN retry_count >= max_retries THEN 'dead_letter'
                    ELSE 'retrying'
                END,
                next_attempt_at = CASE
                    WHEN retry_count >= max_retries THEN next_attempt_at
                    ELSE $2
                END,
                locked_at = NULL,
                locked_by = NULL,
                last_error = 'WhatsApp provider command execution was interrupted before provider reconciliation',
                result_payload = COALESCE(result_payload, '{}'::jsonb) || $3::jsonb,
                provider_state = COALESCE(provider_state, '{}'::jsonb) || $4::jsonb,
                reconciliation_status = 'not_observed',
                dead_lettered_at = CASE
                    WHEN retry_count >= max_retries THEN $5
                    ELSE dead_lettered_at
                END,
                updated_at = $5
            WHERE command_id = $1
              AND status = 'executing'
            RETURNING *
            "#,
        )
        .bind(&command.command_id)
        .bind(next_attempt_at(now, command.retry_count, None))
        .bind(failure_result_payload)
        .bind(failure_provider_state)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let updated = row_to_whatsapp_provider_write_command(row)?;
            mirror_canonical_provider_command_for_pool(pool, &updated).await?;
            recovered.push(updated);
        }
    }

    Ok(recovered)
}

pub(crate) async fn recover_stale_fixture_executing_commands(
    pool: &PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    recover_stale_executing_commands_scoped(pool, now, Some(true), None).await
}

pub(crate) async fn recover_stale_live_executing_commands(
    pool: &PgPool,
    now: DateTime<Utc>,
    account_id: Option<&str>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    recover_stale_executing_commands_scoped(pool, now, Some(false), account_id).await
}

pub(crate) async fn reschedule_failed_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
    error_code: Option<&str>,
    retry_after_seconds: Option<i64>,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let current_retry_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT retry_count
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
          AND status = 'executing'
        "#,
    )
    .bind(command_id)
    .fetch_optional(pool)
    .await?;
    let Some(retry_count) = current_retry_count else {
        return Ok(None);
    };
    let next_attempt_at = next_attempt_at(now, retry_count, retry_after_seconds);
    let failure_result_payload = json!({
        "failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": retry_after_seconds,
            "reported_at": now,
            "reported_via": "runtime_bridge_failed",
        }
    });
    let failure_provider_state = json!({
        "last_failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": retry_after_seconds,
            "reported_at": now,
            "reported_via": "runtime_bridge_failed",
        }
    });
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = CASE
                WHEN retry_count >= max_retries THEN 'dead_letter'
                ELSE 'retrying'
            END,
            next_attempt_at = CASE
                WHEN retry_count >= max_retries THEN next_attempt_at
                ELSE $3
            END,
            locked_at = NULL,
            locked_by = NULL,
            last_error = $4,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $5::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $6::jsonb,
            reconciliation_status = 'not_observed',
            dead_lettered_at = CASE
                WHEN retry_count >= max_retries THEN $2
                ELSE dead_lettered_at
            END,
            updated_at = $2
        WHERE command_id = $1
          AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(next_attempt_at)
    .bind(error_message)
    .bind(failure_result_payload)
    .bind(failure_provider_state)
    .fetch_optional(pool)
    .await?;

    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

pub(crate) async fn dead_letter_failed_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
    error_code: Option<&str>,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let failure_result_payload = json!({
        "failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": null,
            "reported_at": now,
            "reported_via": "runtime_bridge_terminal_failed",
            "retry_policy": "terminal",
        }
    });
    let failure_provider_state = json!({
        "last_failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": null,
            "reported_at": now,
            "reported_via": "runtime_bridge_terminal_failed",
            "retry_policy": "terminal",
        }
    });
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = 'dead_letter',
            next_attempt_at = NULL,
            locked_at = NULL,
            locked_by = NULL,
            last_error = $3,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $4::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $5::jsonb,
            reconciliation_status = 'not_observed',
            dead_lettered_at = $2,
            updated_at = $2
        WHERE command_id = $1
          AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(error_message)
    .bind(failure_result_payload)
    .bind(failure_provider_state)
    .fetch_optional(pool)
    .await?;

    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

pub(crate) async fn record_live_provider_command_submitted(
    pool: &PgPool,
    now: DateTime<Utc>,
    outcome: &WhatsAppProviderCommandExecutionOutcome,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = 'executing',
            locked_at = NULL,
            locked_by = NULL,
            last_error = NULL,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $3::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $4::jsonb,
            reconciliation_status = 'awaiting_provider',
            updated_at = $2
        WHERE command_id = $1
          AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(&outcome.command_id)
    .bind(now)
    .bind(&outcome.result_payload)
    .bind(&outcome.provider_state)
    .fetch_optional(pool)
    .await?;

    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

impl From<WhatsAppProviderWriteCommand> for WhatsAppProviderCommand {
    fn from(command: WhatsAppProviderWriteCommand) -> Self {
        Self {
            command_id: command.command_id,
            account_id: command.account_id,
            command_kind: command.command_kind,
            idempotency_key: command.idempotency_key,
            provider_chat_id: command.provider_chat_id,
            provider_message_id: command.provider_message_id,
            capability_state: command.capability_state,
            action_class: command.action_class,
            confirmation_decision: command.confirmation_decision,
            status: command.status,
            retry_count: command.retry_count,
            max_retries: command.max_retries,
            last_error: command.last_error,
            result_payload: command.result_payload,
            audit_metadata: command.audit_metadata,
            provider_state: command.provider_state,
            reconciliation_status: command.reconciliation_status,
            next_attempt_at: command.next_attempt_at,
            last_attempt_at: command.last_attempt_at,
            provider_observed_at: command.provider_observed_at,
            reconciled_at: command.reconciled_at,
            dead_lettered_at: command.dead_lettered_at,
            completed_at: command.completed_at,
            created_at: command.created_at,
            updated_at: command.updated_at,
        }
    }
}

impl From<&WhatsAppProviderWriteCommand> for WhatsAppProviderExecutableCommand {
    fn from(command: &WhatsAppProviderWriteCommand) -> Self {
        Self {
            command_id: command.command_id.clone(),
            account_id: command.account_id.clone(),
            command_kind: command.command_kind.clone(),
            idempotency_key: command.idempotency_key.clone(),
            provider_chat_id: command.provider_chat_id.clone(),
            provider_message_id: command.provider_message_id.clone(),
            payload: command.payload.clone(),
            target_ref: command.target_ref.clone(),
            audit_metadata: command.audit_metadata.clone(),
            provider_state: command.provider_state.clone(),
            media_bytes: None,
            media_download_ref: None,
            api_access_token: None,
        }
    }
}

fn clamp_limit(limit: i64) -> i64 {
    limit.clamp(1, 200)
}

fn runtime_blockers(runtime_kind: &str, last_error: Option<&str>) -> Vec<String> {
    let mut blockers = Vec::new();
    if runtime_kind != "fixture" {
        blockers.push("live_whatsapp_runtime_blocked".to_owned());
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push(error.trim().to_owned());
    }
    blockers
}

fn qr_pair_code_blockers(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
) -> Vec<String> {
    let mut blockers = vec![
        "whatsapp_qr_pairing_requires_visible_runtime".to_owned(),
        "live_whatsapp_runtime_blocked".to_owned(),
    ];
    if runtime_kind == "fixture" {
        blockers.push("fixture_runtime_cannot_link_live_accounts".to_owned());
    }
    if runtime_kind != "fixture" && !provider_shape_runtime_feature_enabled(provider_shape) {
        blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
    }
    blockers
}

fn provider_command_blockers(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    session_restore_available: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if runtime_kind == "fixture" {
        blockers.push("fixture_runtime_does_not_execute_provider_commands".to_owned());
    } else {
        blockers.push("live_whatsapp_runtime_blocked".to_owned());
        blockers.push("whatsapp_live_provider_execution_missing".to_owned());
        if !provider_shape_runtime_feature_enabled(provider_shape) {
            blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
        }
    }
    if !session_restore_available {
        blockers.push("whatsapp_session_restore_unavailable".to_owned());
    }
    blockers
}

fn whatsapp_account_lifecycle_state(account: &ProviderAccount) -> &str {
    account
        .config
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("created")
}

fn runtime_runtime_available(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    status: &str,
) -> bool {
    runtime_kind != "fixture"
        && runtime_kind != "live_blocked"
        && provider_shape_runtime_feature_enabled(provider_shape)
        && matches!(status, "linked" | "available" | "syncing" | "degraded")
}

fn media_transfer_available(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    status: &str,
) -> bool {
    runtime_runtime_available(runtime_kind, provider_shape, status)
        && matches!(status, "available" | "degraded")
}

fn runtime_health_status(status: &WhatsAppRuntimeStatus) -> &'static str {
    if !status.runtime_blockers.is_empty() {
        return "blocked";
    }
    if status.status == "available" && status.live_runtime_available && status.live_send_available {
        return "available";
    }
    "degraded"
}

fn runtime_status_blockers(
    status: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    runtime_kind: &str,
    session_restore_available: bool,
    last_error: Option<&str>,
) -> Vec<String> {
    let mut blockers = Vec::new();
    match status {
        "link_required" | "created" => {
            if provider_shape == WhatsAppProviderRuntimeShape::BusinessCloud {
                blockers.push("whatsapp_business_cloud_setup_required".to_owned());
            } else {
                blockers.push("whatsapp_session_link_required".to_owned());
            }
        }
        "qr_pending" | "pair_code_pending" => {
            blockers.extend(qr_pair_code_blockers(runtime_kind, provider_shape));
        }
        "revoked" => {
            blockers.push("whatsapp_session_revoked".to_owned());
        }
        "removed" => {
            blockers.push("whatsapp_account_removed".to_owned());
        }
        "blocked" => {
            blockers.push("live_whatsapp_runtime_blocked".to_owned());
        }
        _ => {}
    }
    if status == "link_required"
        && runtime_kind != "fixture"
        && provider_shape != WhatsAppProviderRuntimeShape::BusinessCloud
    {
        blockers.push("whatsapp_visible_runtime_required".to_owned());
    }
    if runtime_kind != "fixture" && !provider_shape_runtime_feature_enabled(provider_shape) {
        blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
    }
    if !session_restore_available
        && !matches!(
            status,
            "revoked" | "removed" | "blocked" | "available" | "linked"
        )
    {
        blockers.push("whatsapp_session_restore_unavailable".to_owned());
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push("whatsapp_runtime_error_present".to_owned());
    }
    blockers
}

fn provider_shape_runtime_feature_enabled(provider_shape: WhatsAppProviderRuntimeShape) -> bool {
    match provider_shape {
        WhatsAppProviderRuntimeShape::WebCompanion => true,
        WhatsAppProviderRuntimeShape::NativeMultiDevice => {
            whatsapp_native_md_runtime_feature_enabled()
        }
        WhatsAppProviderRuntimeShape::BusinessCloud => {
            whatsapp_business_cloud_runtime_feature_enabled()
        }
    }
}

fn authorized_session_runtime_kind(account: &ProviderAccount, extra: &Value) -> String {
    extra
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| account_runtime_kind(account))
}

#[cfg(test)]
mod runtime_feature_tests {
    use super::*;

    #[test]
    fn whatsapp_runtime_feature_gates_default_to_expected_shapes() {
        assert!(provider_shape_runtime_feature_enabled(
            WhatsAppProviderRuntimeShape::WebCompanion
        ));
        assert_eq!(
            provider_shape_runtime_feature_enabled(WhatsAppProviderRuntimeShape::NativeMultiDevice),
            whatsapp_native_md_runtime_feature_enabled()
        );
        assert_eq!(
            provider_shape_runtime_feature_enabled(WhatsAppProviderRuntimeShape::BusinessCloud),
            whatsapp_business_cloud_runtime_feature_enabled()
        );
    }

    #[test]
    fn whatsapp_runtime_feature_blockers_match_provider_shapes() {
        assert_eq!(
            provider_shape_runtime_feature_blocker(WhatsAppProviderRuntimeShape::WebCompanion),
            "whatsapp_web_runtime_feature_unavailable"
        );
        assert_eq!(
            provider_shape_runtime_feature_blocker(WhatsAppProviderRuntimeShape::NativeMultiDevice),
            native_md::native_md_runtime_feature_blocker()
        );
        assert_eq!(
            provider_shape_runtime_feature_blocker(WhatsAppProviderRuntimeShape::BusinessCloud),
            business_cloud::business_cloud_runtime_feature_blocker()
        );
    }

    #[test]
    fn whatsapp_retry_delay_defaults_to_exponential_backoff_and_caps() {
        assert_eq!(retry_delay_seconds(0, None), 30);
        assert_eq!(retry_delay_seconds(1, None), 30);
        assert_eq!(retry_delay_seconds(2, None), 60);
        assert_eq!(retry_delay_seconds(3, None), 120);
        assert_eq!(retry_delay_seconds(6, None), 900);
        assert_eq!(retry_delay_seconds(20, None), 900);
    }

    #[test]
    fn whatsapp_retry_delay_honors_runtime_hint_with_clamp() {
        assert_eq!(retry_delay_seconds(4, Some(5)), 5);
        assert_eq!(retry_delay_seconds(4, Some(5_000)), 900);
        assert_eq!(retry_delay_seconds(4, Some(-10)), 1);
    }
}

fn provider_shape_runtime_feature_blocker(
    provider_shape: WhatsAppProviderRuntimeShape,
) -> &'static str {
    match provider_shape {
        WhatsAppProviderRuntimeShape::WebCompanion => "whatsapp_web_runtime_feature_unavailable",
        WhatsAppProviderRuntimeShape::NativeMultiDevice => {
            native_md::native_md_runtime_feature_blocker()
        }
        WhatsAppProviderRuntimeShape::BusinessCloud => {
            business_cloud::business_cloud_runtime_feature_blocker()
        }
    }
}

fn next_attempt_at(
    now: DateTime<Utc>,
    retry_count: i32,
    retry_after_seconds: Option<i64>,
) -> DateTime<Utc> {
    now + chrono::Duration::seconds(retry_delay_seconds(retry_count, retry_after_seconds))
}

fn retry_delay_seconds(retry_count: i32, retry_after_seconds: Option<i64>) -> i64 {
    if let Some(seconds) = retry_after_seconds {
        return seconds.clamp(1, RETRY_MAX_DELAY_SECONDS);
    }

    let retry_index = retry_count.saturating_sub(1).clamp(0, 20) as u32;
    let multiplier = 1_i64.checked_shl(retry_index).unwrap_or(i64::MAX);
    RETRY_BASE_DELAY_SECONDS
        .saturating_mul(multiplier)
        .min(RETRY_MAX_DELAY_SECONDS)
}

fn setup_id(prefix: &str, account_id: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        account_id.trim(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    )
}

fn fixture_whatsapp_qr_payload(account_id: &str, setup_id: &str) -> String {
    format!(
        "hermes-whatsapp-fixture://link?account_id={}&setup_id={}",
        account_id.trim(),
        setup_id.trim()
    )
}

fn render_fixture_whatsapp_qr_svg(payload: &str) -> Result<String, WhatsappWebError> {
    let code = QrCode::new(payload.as_bytes()).map_err(|error| {
        WhatsappWebError::InvalidRequest(format!("failed to encode QR: {error}"))
    })?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}

fn fixture_whatsapp_pair_code(account_id: &str, phone_number: &str, setup_id: &str) -> String {
    let seed = short_hash(&format!(
        "{}:{}:{}",
        account_id.trim(),
        phone_number.trim(),
        setup_id.trim()
    ))
    .to_uppercase();
    format!("{}-{}", &seed[..4], &seed[4..8])
}

fn validated_or_generated_command_id(
    command_id: &Option<String>,
) -> Result<String, WhatsappWebError> {
    match command_id {
        Some(value) => validate_non_empty("command_id", value),
        None => Ok(new_whatsapp_command_id()),
    }
}

fn new_whatsapp_command_id() -> String {
    let now = Utc::now();
    format!(
        "wacmd_{}_{}",
        now.timestamp_millis(),
        short_hash(&format!(
            "whatsapp-command-{}",
            now.timestamp_nanos_opt().unwrap_or(0)
        ))
    )
}

fn whatsapp_text_preview_hash(text: &str) -> String {
    format!("sha256:{}", short_hash(text.trim()))
}

fn short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

fn whatsapp_session_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        ProviderAccountSecretPurpose::WhatsappWebSessionKey.as_str()
    )
}

pub(crate) fn whatsapp_business_cloud_access_token_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        ProviderAccountSecretPurpose::WhatsappBusinessCloudAccessToken.as_str()
    )
}

pub(crate) fn whatsapp_business_cloud_app_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        ProviderAccountSecretPurpose::WhatsappBusinessCloudAppSecret.as_str()
    )
}

pub(crate) fn whatsapp_business_cloud_webhook_verify_token_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        ProviderAccountSecretPurpose::WhatsappBusinessCloudWebhookVerifyToken.as_str()
    )
}

pub(crate) fn whatsapp_native_md_media_download_secret_ref(
    account_id: &str,
    fingerprint: &str,
) -> String {
    let suffix = fingerprint
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(fingerprint.trim())
        .chars()
        .take(32)
        .collect::<String>();
    format!(
        "secret:provider-account:{}:whatsapp_media_download_ref:{}",
        account_id.trim(),
        suffix
    )
}

fn session_secret_metadata(account: &ProviderAccount, extra: &Value) -> Value {
    let mut metadata = json!({
        "provider": account.provider_kind.as_str(),
        "provider_shape": account_provider_shape(account, WhatsAppProviderRuntimeShape::WebCompanion).as_str(),
        "account_id": account.account_id,
        "secret_purpose": ProviderAccountSecretPurpose::WhatsappWebSessionKey.as_str(),
        "runtime": account_runtime_kind(account),
    });
    if let (Some(metadata_object), Some(extra_object)) =
        (metadata.as_object_mut(), extra.as_object())
    {
        for (key, value) in extra_object {
            metadata_object.insert(key.clone(), value.clone());
        }
    }
    metadata
}

fn ensure_qr_pairing_supported(
    provider_shape: WhatsAppProviderRuntimeShape,
    operation: &'static str,
) -> Result<(), WhatsappWebError> {
    if provider_shape == WhatsAppProviderRuntimeShape::BusinessCloud {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "WhatsApp provider shape `{}` does not support `{operation}`",
            provider_shape.as_str()
        )));
    }
    Ok(())
}

fn ensure_session_secret_supported(
    provider_shape: WhatsAppProviderRuntimeShape,
    operation: &'static str,
) -> Result<(), WhatsappWebError> {
    if provider_shape == WhatsAppProviderRuntimeShape::BusinessCloud {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "WhatsApp provider shape `{}` does not support `{operation}`",
            provider_shape.as_str()
        )));
    }
    Ok(())
}

fn ensure_provider_command_supported(
    provider_shape: WhatsAppProviderRuntimeShape,
    command_kind: &str,
) -> Result<(), WhatsappWebError> {
    if provider_shape == WhatsAppProviderRuntimeShape::BusinessCloud {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "WhatsApp provider shape `{}` does not support personal command `{command_kind}`",
            provider_shape.as_str()
        )));
    }
    Ok(())
}

fn provider_request_id_matches_observed_receipt(
    command: &WhatsAppProviderWriteCommand,
    receipt: &NewWhatsappWebReceipt,
) -> bool {
    let observed_provider_message_id = receipt.provider_message_id.trim();
    if observed_provider_message_id.is_empty() {
        return false;
    }

    command.provider_message_id.as_deref() == Some(observed_provider_message_id)
        || [
            json_string_at(
                &command.provider_state,
                &["business_cloud", "provider_request_id"],
            ),
            json_string_at(
                &command.provider_state,
                &[
                    "business_cloud",
                    "provider_observed_completion_target",
                    "provider_message_id",
                ],
            ),
            json_string_at(
                &command.provider_state,
                &["native_md", "provider_request_id"],
            ),
            json_string_at(
                &command.provider_state,
                &[
                    "native_md",
                    "provider_observed_completion_target",
                    "provider_message_id",
                ],
            ),
            json_string_at(
                &command.result_payload,
                &["provider_submission", "provider_request_id"],
            ),
            json_string_at(
                &command.result_payload,
                &[
                    "provider_submission",
                    "provider_observed_completion_target",
                    "provider_message_id",
                ],
            ),
        ]
        .into_iter()
        .flatten()
        .any(|provider_request_id| provider_request_id == observed_provider_message_id)
}

fn provider_request_id_matches_observed_media(
    command: &WhatsAppProviderWriteCommand,
    media: &NewWhatsappWebMedia,
) -> bool {
    let observed_provider_message_id = media.provider_message_id.trim();
    if observed_provider_message_id.is_empty() {
        return false;
    }

    [
        json_string_at(
            &command.provider_state,
            &["native_md", "provider_request_id"],
        ),
        json_string_at(
            &command.provider_state,
            &[
                "native_md",
                "provider_observed_completion_target",
                "provider_message_id",
            ],
        ),
        json_string_at(
            &command.result_payload,
            &["provider_submission", "provider_request_id"],
        ),
        json_string_at(
            &command.result_payload,
            &[
                "provider_submission",
                "provider_observed_completion_target",
                "provider_message_id",
            ],
        ),
    ]
    .into_iter()
    .flatten()
    .any(|provider_request_id| provider_request_id == observed_provider_message_id)
}

fn json_string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, WhatsappWebError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_reconciliation_matches_native_provider_request_id() {
        let now = Utc::now();
        let command = WhatsAppProviderWriteCommand {
            command_id: "cmd-media".to_owned(),
            account_id: "acct".to_owned(),
            command_kind: "send_media".to_owned(),
            idempotency_key: "media-key".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: None,
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            status: "executing".to_owned(),
            retry_count: 1,
            max_retries: 3,
            last_error: None,
            payload: json!({"blob_id": "different-blob"}),
            target_ref: json!({}),
            result_payload: json!({
                "provider_submission": {
                    "provider_request_id": "wamid.native.1"
                }
            }),
            audit_metadata: json!({}),
            provider_state: json!({
                "native_md": {
                    "provider_request_id": "wamid.native.1"
                }
            }),
            reconciliation_status: "awaiting_provider".to_owned(),
            next_attempt_at: None,
            last_attempt_at: Some(now),
            provider_observed_at: None,
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        };
        let media = NewWhatsappWebMedia {
            account_id: "acct".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "wamid.native.1".to_owned(),
            provider_attachment_id: "attachment".to_owned(),
            filename: None,
            content_type: "image/png".to_owned(),
            size_bytes: 42,
            sha256: "sha256:input".to_owned(),
            storage_kind: "local_fs".to_owned(),
            storage_path: "observed-blob".to_owned(),
            import_batch_id: "batch".to_owned(),
            observed_at: now,
        };

        assert!(provider_request_id_matches_observed_media(&command, &media));
    }

    #[test]
    fn receipt_reconciliation_matches_business_cloud_provider_request_id() {
        let now = Utc::now();
        let command = WhatsAppProviderWriteCommand {
            command_id: "cmd-business-cloud".to_owned(),
            account_id: "acct".to_owned(),
            command_kind: "send_template".to_owned(),
            idempotency_key: "template-key".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: None,
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            status: "executing".to_owned(),
            retry_count: 1,
            max_retries: 3,
            last_error: None,
            payload: json!({}),
            target_ref: json!({}),
            result_payload: json!({
                "provider_submission": {
                    "provider_request_id": "wamid.business.1",
                    "provider_observed_completion_target": {
                        "provider_message_id": "wamid.business.1",
                        "match_policy": "provider_request_id_equals_observed_receipt_provider_message_id"
                    }
                }
            }),
            audit_metadata: json!({}),
            provider_state: json!({
                "business_cloud": {
                    "provider_request_id": "wamid.business.1",
                    "provider_observed_completion_target": {
                        "provider_message_id": "wamid.business.1"
                    }
                }
            }),
            reconciliation_status: "awaiting_provider".to_owned(),
            next_attempt_at: None,
            last_attempt_at: Some(now),
            provider_observed_at: None,
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        };
        let receipt = NewWhatsappWebReceipt {
            account_id: "acct".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "wamid.business.1".to_owned(),
            delivery_state:
                crate::integrations::whatsapp::client::WhatsappWebDeliveryState::Delivered,
            import_batch_id: "batch".to_owned(),
            observed_at: now,
        };

        assert!(provider_request_id_matches_observed_receipt(
            &command, &receipt
        ));
    }
}
