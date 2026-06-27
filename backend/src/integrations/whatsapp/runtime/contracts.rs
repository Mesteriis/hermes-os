use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
};
use crate::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use crate::vault::HostVault;

pub type WhatsAppProviderRuntimeFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, WhatsappWebError>> + Send + 'a>>;

pub type WhatsAppProviderCommandExecutionFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    WhatsAppProviderCommandExecutionOutcome,
                    WhatsAppProviderCommandExecutionError,
                >,
            > + Send
            + 'a,
    >,
>;

pub type WhatsAppRuntimeEventSinkFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), WhatsAppRuntimeEventSinkError>> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppProviderRuntimeShape {
    WebCompanion,
    NativeMultiDevice,
    BusinessCloud,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeBridgeDispatch {
    pub endpoint_path: &'static str,
    pub request_kind: &'static str,
    pub observed_source: &'static str,
}

impl WhatsAppRuntimeBridgeDispatch {
    pub fn new(
        endpoint_path: &'static str,
        request_kind: &'static str,
        observed_source: &'static str,
    ) -> Self {
        Self {
            endpoint_path,
            request_kind,
            observed_source,
        }
    }

    pub fn assert_runtime_bridge_contract(self) {
        debug_assert!(
            self.endpoint_path
                .starts_with("/api/v1/integrations/whatsapp/runtime-bridge/")
        );
        debug_assert!(!self.request_kind.trim().is_empty());
        debug_assert!(
            self.observed_source
                .starts_with("provider_observed.runtime_bridge_")
        );
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WhatsAppSanitizedRuntimeEventDto {
    pub account_id: String,
    pub provider_event_id: String,
    pub provider_shape: &'static str,
    pub runtime_driver: &'static str,
    pub provider_event_name: &'static str,
    pub event_family: &'static str,
    pub raw_record_kind: &'static str,
    pub raw_signal_event_kind: &'static str,
    pub accepted_event_kind: &'static str,
    pub source_fingerprint_seed: String,
    pub bridge_dispatch: WhatsAppRuntimeBridgeDispatch,
    pub metadata: Value,
}

impl WhatsAppSanitizedRuntimeEventDto {
    pub fn assert_event_spine_contract(&self) {
        debug_assert!(!self.account_id.trim().is_empty());
        debug_assert!(!self.provider_event_id.trim().is_empty());
        debug_assert_eq!(self.provider_shape, "whatsapp_native_md");
        debug_assert!(!self.runtime_driver.trim().is_empty());
        debug_assert!(!self.provider_event_name.trim().is_empty());
        debug_assert!(!self.event_family.trim().is_empty());
        debug_assert!(self.raw_record_kind.starts_with("whatsapp_web_"));
        debug_assert!(
            self.raw_signal_event_kind
                .starts_with("signal.raw.whatsapp.")
                && self.raw_signal_event_kind.ends_with(".observed")
        );
        debug_assert!(
            self.accepted_event_kind
                .starts_with("signal.accepted.whatsapp.")
        );
        debug_assert!(
            self.source_fingerprint_seed
                .starts_with("source_fingerprint:v5:")
        );
        self.bridge_dispatch.assert_runtime_bridge_contract();
        debug_assert_eq!(
            self.metadata.get("payload_policy").and_then(Value::as_str),
            Some("sanitized_metadata_only")
        );
        debug_assert_eq!(
            self.metadata
                .get("session_material")
                .and_then(Value::as_str),
            Some("excluded")
        );
        debug_assert_eq!(
            self.metadata
                .get("raw_provider_payload")
                .and_then(Value::as_str),
            Some("excluded")
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppRuntimeEventSinkError {
    pub code: &'static str,
}

impl WhatsAppRuntimeEventSinkError {
    pub fn new(code: &'static str) -> Self {
        Self { code }
    }
}

pub trait WhatsAppRuntimeEventSink: Send + Sync {
    fn accept<'a>(
        &'a self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> WhatsAppRuntimeEventSinkFuture<'a>;
}

impl WhatsAppProviderRuntimeShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebCompanion => "whatsapp_web_companion",
            Self::NativeMultiDevice => "whatsapp_native_md",
            Self::BusinessCloud => "whatsapp_business_cloud",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeStartRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeStopRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRevokeRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRelinkRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRemoveRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppQrLinkStartRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppPairCodeStartRequest {
    pub account_id: String,
    pub phone_number: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppTextSendRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppReplyRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub reply_to_provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppForwardRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_provider_chat_id: String,
    pub from_provider_message_id: String,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppEditRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppDeleteRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub confirmation_decision: Option<String>,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppReactionRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reaction_emoji: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppConversationCommandRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub confirmation_decision: Option<String>,
    pub invite_link: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppStatusPublishRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppVoiceNoteSendRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub attachment_id: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppMediaUploadRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub attachment_id: Option<String>,
    pub blob_id: String,
    pub media_type: String,
    pub caption: Option<String>,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppMediaDownloadRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: Option<String>,
    pub provider_media_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub fixture_runtime: bool,
    pub live_runtime_available: bool,
    pub live_send_available: bool,
    pub qr_pairing_available: bool,
    pub pair_code_available: bool,
    pub media_download_available: bool,
    pub media_upload_available: bool,
    pub session_restore_available: bool,
    pub session_secret_ref: Option<String>,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeHealth {
    pub account_id: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub healthy: bool,
    pub checks: Value,
    pub checked_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeRemoveResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub removed: bool,
    pub unbound_secret_refs: Vec<String>,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppQrLinkSession {
    pub account_id: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub setup_id: String,
    pub qr_svg: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub runtime_blockers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppPairCodeSession {
    pub account_id: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub setup_id: String,
    pub phone_number: String,
    pub pair_code: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub runtime_blockers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppAuthorizedSessionCredentialWrite {
    pub account_id: String,
    pub session_material: String,
    #[serde(default = "default_session_secret_kind")]
    pub secret_kind: SecretKind,
    #[serde(default = "default_session_secret_label")]
    pub label: String,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

impl WhatsAppAuthorizedSessionCredentialWrite {
    pub fn new(account_id: impl Into<String>, session_material: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            session_material: session_material.into(),
            secret_kind: SecretKind::Other,
            label: "WhatsApp session credential".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn secret_kind(mut self, secret_kind: SecretKind) -> Self {
        self.secret_kind = secret_kind;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppCredentialBinding {
    pub secret_purpose: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderCommandResponse {
    pub command_id: String,
    pub idempotency_key: String,
    pub command_kind: String,
    pub account_id: String,
    pub provider_kind: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub provider_chat_id: String,
    pub provider_message_id: Option<String>,
    pub status: String,
    pub durable_status: String,
    pub delivery_state: String,
    pub session_restore_available: bool,
    pub rendered_preview_hash: Option<String>,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderCommand {
    pub command_id: String,
    pub account_id: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_chat_id: String,
    pub provider_message_id: Option<String>,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub result_payload: Value,
    pub audit_metadata: Value,
    pub provider_state: Value,
    pub reconciliation_status: String,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub provider_observed_at: Option<DateTime<Utc>>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub dead_lettered_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct WhatsAppProviderInMemoryMediaBytes {
    bytes: Vec<u8>,
}

impl WhatsAppProviderInMemoryMediaBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn clone_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

impl fmt::Debug for WhatsAppProviderInMemoryMediaBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WhatsAppProviderInMemoryMediaBytes")
            .field("size_bytes", &self.bytes.len())
            .field("payload", &"redacted")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct WhatsAppProviderMediaDownloadRef {
    pub secret_ref: String,
    pub provider_media_ref_fingerprint: String,
    pub media_type: String,
    pub content_type: String,
    pub file_length: u64,
    pub file_sha256: Vec<u8>,
    pub file_enc_sha256: Vec<u8>,
    pub direct_path: String,
    pub static_url: Option<String>,
    pub media_key: Vec<u8>,
    pub media_key_timestamp: Option<i64>,
}

impl fmt::Debug for WhatsAppProviderMediaDownloadRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WhatsAppProviderMediaDownloadRef")
            .field("secret_ref", &self.secret_ref)
            .field(
                "provider_media_ref_fingerprint",
                &self.provider_media_ref_fingerprint,
            )
            .field("media_type", &self.media_type)
            .field("content_type", &self.content_type)
            .field("file_length", &self.file_length)
            .field("file_sha256_bytes", &self.file_sha256.len())
            .field("file_enc_sha256_bytes", &self.file_enc_sha256.len())
            .field("direct_path", &"redacted")
            .field("static_url", &self.static_url.as_ref().map(|_| "redacted"))
            .field("media_key_bytes", &self.media_key.len())
            .field("media_key", &"redacted")
            .field("media_key_timestamp", &self.media_key_timestamp)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct WhatsAppProviderApiAccessToken {
    secret_ref: String,
    token: String,
}

impl WhatsAppProviderApiAccessToken {
    pub fn new(secret_ref: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            secret_ref: secret_ref.into(),
            token: token.into(),
        }
    }

    pub fn secret_ref(&self) -> &str {
        &self.secret_ref
    }

    pub fn expose_for_runtime(&self) -> &str {
        &self.token
    }
}

impl fmt::Debug for WhatsAppProviderApiAccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WhatsAppProviderApiAccessToken")
            .field("secret_ref", &self.secret_ref)
            .field("token", &"redacted")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderExecutableCommand {
    pub command_id: String,
    pub account_id: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_chat_id: String,
    pub provider_message_id: Option<String>,
    pub payload: Value,
    pub target_ref: Value,
    pub audit_metadata: Value,
    pub provider_state: Value,
    #[serde(skip_serializing)]
    pub media_bytes: Option<WhatsAppProviderInMemoryMediaBytes>,
    #[serde(skip_serializing)]
    pub media_download_ref: Option<WhatsAppProviderMediaDownloadRef>,
    #[serde(skip_serializing)]
    pub api_access_token: Option<WhatsAppProviderApiAccessToken>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderCommandExecutionOutcome {
    pub command_id: String,
    pub provider_request_id: Option<String>,
    pub result_payload: Value,
    pub provider_state: Value,
    #[serde(skip_serializing)]
    pub downloaded_media_bytes: Option<WhatsAppProviderInMemoryMediaBytes>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderCommandExecutionError {
    pub error_message: String,
    pub error_code: Option<String>,
    pub retry_after_seconds: Option<i64>,
}

impl WhatsAppProviderCommandExecutionError {
    pub fn new(
        error_code: impl Into<String>,
        error_message: impl Into<String>,
        retry_after_seconds: Option<i64>,
    ) -> Self {
        Self {
            error_message: error_message.into(),
            error_code: Some(error_code.into()),
            retry_after_seconds,
        }
    }

    pub fn unsupported(command_kind: &str) -> Self {
        Self::new(
            "whatsapp_live_provider_execution_missing",
            format!(
                "live WhatsApp provider command execution is not available for `{command_kind}`"
            ),
            None,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppProviderCommandListResponse {
    pub items: Vec<WhatsAppProviderCommand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppCommandDeadLetterRequest {
    pub reason: String,
}

pub trait WhatsAppProviderRuntime: Send + Sync {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape;

    fn runtime_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus>;

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus>;

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus>;

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus>;

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus>;

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse>;

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth>;

    fn start_qr_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppQrLinkStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppQrLinkSession>;

    fn start_pair_code_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppPairCodeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppPairCodeSession>;

    fn request_send_text<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppTextSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_reply<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReplyRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_forward<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppForwardRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_edit<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppEditRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_delete<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppDeleteRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_react<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_unreact<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_media_upload<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaUploadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_media_download<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaDownloadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_mark_read<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_mark_unread<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_archive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_unarchive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_mute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_unmute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_pin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_unpin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_join_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_leave_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_publish_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppStatusPublishRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn request_send_voice_note<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppVoiceNoteSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse>;

    fn execute_live_provider_command<'a>(
        &'a self,
        command: &'a WhatsAppProviderExecutableCommand,
    ) -> WhatsAppProviderCommandExecutionFuture<'a> {
        Box::pin(async move {
            Err(WhatsAppProviderCommandExecutionError::unsupported(
                &command.command_kind,
            ))
        })
    }

    fn list_provider_commands<'a>(
        &'a self,
        account_id: &'a str,
        provider_chat_id: Option<&'a str>,
        provider_message_id: Option<&'a str>,
        command_kinds: &'a [String],
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse>;

    fn manual_retry_provider_command<'a>(
        &'a self,
        command_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>>;

    fn dead_letter_provider_command<'a>(
        &'a self,
        command_id: &'a str,
        reason: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>>;

    fn store_authorized_session_credential<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding>;

    fn setup_fixture_account<'a>(
        &'a self,
        request: &'a WhatsappWebAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse>;

    fn setup_live_blocked_account<'a>(
        &'a self,
        request: &'a WhatsappLiveAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse>;

    fn list_sessions<'a>(
        &'a self,
        account_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>>;

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        provider_chat_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>>;

    fn ingest_fixture_message<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessage>;

    fn reconcile_fixture_message_commands<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_reaction<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReaction>;

    fn reconcile_fixture_reaction_commands<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_media<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMedia>;

    fn reconcile_fixture_media_commands<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_status<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatus>;

    fn ingest_fixture_status_view<'a>(
        &'a self,
        status_view: &'a NewWhatsappWebStatusView,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusView>;

    fn ingest_fixture_status_delete<'a>(
        &'a self,
        status_delete: &'a NewWhatsappWebStatusDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusDelete>;

    fn ingest_fixture_presence<'a>(
        &'a self,
        presence: &'a NewWhatsappWebPresence,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedPresence>;

    fn ingest_fixture_call<'a>(
        &'a self,
        call: &'a NewWhatsappWebCall,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedCall>;

    fn ingest_fixture_runtime_event<'a>(
        &'a self,
        runtime_event: &'a NewWhatsappWebRuntimeEvent,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedRuntimeEvent>;

    fn reconcile_fixture_status_commands<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_dialog<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedDialog>;

    fn reconcile_fixture_dialog_commands<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_participant<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedParticipant>;

    fn reconcile_fixture_participant_commands<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_message_update<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageUpdate>;

    fn reconcile_fixture_message_update_commands<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_message_delete<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageDelete>;

    fn reconcile_fixture_message_delete_commands<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;

    fn ingest_fixture_receipt<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReceipt>;

    fn reconcile_fixture_receipt_commands<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>>;
}

fn default_session_secret_kind() -> SecretKind {
    SecretKind::Other
}

fn default_session_secret_label() -> String {
    "WhatsApp session credential".to_owned()
}

fn default_json_object() -> Value {
    json!({})
}
