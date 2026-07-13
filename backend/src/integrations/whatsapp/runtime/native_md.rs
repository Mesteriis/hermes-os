use hermes_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretPurpose,
};
use hermes_communications_api::accounts::{ProviderAccount, ProviderSecretBindingCommandPort};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "whatsapp-native-md-runtime")]
use base64::Engine as _;
#[cfg(feature = "whatsapp-native-md-runtime")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
#[cfg(feature = "whatsapp-native-md-runtime")]
use std::collections::BTreeMap;
#[cfg(feature = "whatsapp-native-md-runtime")]
use std::str::FromStr;

use super::{
    ShapedWhatsAppProviderRuntime, WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest,
    WhatsAppProviderCommandExecutionError, WhatsAppProviderCommandExecutionOutcome,
    WhatsAppProviderExecutableCommand, WhatsAppProviderInMemoryMediaBytes,
    WhatsAppProviderMediaDownloadRef, WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape,
    WhatsAppQrLinkSession, WhatsAppQrLinkStartRequest, WhatsAppRuntimeBridgeDispatch,
    WhatsAppRuntimeEventSink, WhatsAppRuntimeEventSinkError, WhatsAppRuntimeHealth,
    WhatsAppRuntimeStartRequest, WhatsAppRuntimeStatus, WhatsAppSanitizedRuntimeEventDto,
    WhatsappWebError, WhatsappWebStore,
};
use crate::platform::communications::ProviderChannelMessageLookupPort;
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::HostVault;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdRuntimeCommandChannel {
    DurableOutbox,
}

impl NativeMdRuntimeCommandChannel {
    fn as_str(self) -> &'static str {
        match self {
            Self::DurableOutbox => "durable_provider_command_outbox",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdRuntimeEventSink {
    SignalHubRawEvidence,
}

impl NativeMdRuntimeEventSink {
    fn as_str(self) -> &'static str {
        match self {
            Self::SignalHubRawEvidence => "signal_hub_raw_evidence",
        }
    }
}

const NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_WAIT_SECONDS: u64 = 10;
const NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS: i64 = 180;
const NATIVE_MD_RECONNECT_BASE_DELAY_SECONDS: i64 = 5;
const NATIVE_MD_RECONNECT_MAX_DELAY_SECONDS: i64 = 300;
const NATIVE_MD_RECONNECT_MAX_ATTEMPTS: u32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdTransientAuthArtifactKind {
    QrCode,
    PairCode,
}

#[derive(Clone, Debug, PartialEq)]
struct NativeMdTransientAuthArtifact {
    kind: NativeMdTransientAuthArtifactKind,
    value: String,
    expires_at: DateTime<Utc>,
}

impl NativeMdTransientAuthArtifact {
    fn new(kind: NativeMdTransientAuthArtifactKind, value: String, timeout: Duration) -> Self {
        Self {
            kind,
            value,
            expires_at: native_md_auth_artifact_expires_at(timeout),
        }
    }
}

fn native_md_auth_artifact_expires_at(timeout: Duration) -> DateTime<Utc> {
    let effective_timeout = if timeout.as_secs() == 0 {
        Duration::from_secs(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS as u64)
    } else {
        timeout
    };
    let ttl = chrono::Duration::from_std(effective_timeout).unwrap_or_else(|_| {
        chrono::Duration::seconds(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS)
    });
    Utc::now() + ttl
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Default)]
struct NativeMdTransientAuthArtifacts {
    state: Arc<tokio::sync::Mutex<HashMap<String, NativeMdTransientAuthArtifact>>>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdTransientAuthArtifacts {
    fn new() -> Self {
        Self::default()
    }

    async fn clear(&self, account_id: &str) {
        self.state.lock().await.remove(account_id);
    }

    async fn record_event(&self, account_id: &str, event: &wa_rs::types::events::Event) {
        use wa_rs::types::events::Event;

        let artifact = match event {
            Event::PairingQrCode { code, timeout } => Some(NativeMdTransientAuthArtifact::new(
                NativeMdTransientAuthArtifactKind::QrCode,
                code.clone(),
                *timeout,
            )),
            Event::PairingCode { code, timeout } => Some(NativeMdTransientAuthArtifact::new(
                NativeMdTransientAuthArtifactKind::PairCode,
                code.clone(),
                *timeout,
            )),
            _ => None,
        };

        if let Some(artifact) = artifact {
            self.state
                .lock()
                .await
                .insert(account_id.to_owned(), artifact);
        }
    }

    async fn wait_for(
        &self,
        account_id: &str,
        kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        let deadline = tokio::time::Instant::now()
            + Duration::from_secs(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_WAIT_SECONDS);
        loop {
            if let Some(artifact) = self.take_current(account_id, kind).await {
                return Some(artifact);
            }
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    async fn take_current(
        &self,
        account_id: &str,
        kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        let mut state = self.state.lock().await;
        let artifact = state.get(account_id).cloned()?;
        if artifact.expires_at <= Utc::now() {
            state.remove(account_id);
            return None;
        }
        if artifact.kind != kind {
            return None;
        }
        state.remove(account_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdRuntimeLifecycleSnapshot {
    lifecycle_state: &'static str,
    runtime_status: &'static str,
    severity: &'static str,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    next_reconnect_at: Option<DateTime<Utc>>,
    last_error_code: Option<String>,
    last_event_kind: &'static str,
    updated_at: DateTime<Utc>,
}

impl NativeMdRuntimeLifecycleSnapshot {
    #[allow(clippy::too_many_arguments)]
    fn new(
        lifecycle_state: &'static str,
        runtime_status: &'static str,
        severity: &'static str,
        reconnect_attempts: u32,
        next_reconnect_at: Option<DateTime<Utc>>,
        last_error_code: Option<String>,
        last_event_kind: &'static str,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            lifecycle_state,
            runtime_status,
            severity,
            reconnect_attempts,
            max_reconnect_attempts: NATIVE_MD_RECONNECT_MAX_ATTEMPTS,
            next_reconnect_at,
            last_error_code,
            last_event_kind,
            updated_at,
        }
    }

    fn stopped(now: DateTime<Utc>) -> Self {
        Self::new(
            "stopped",
            "stopped",
            "info",
            0,
            None,
            None,
            "runtime.actor.stopped",
            now,
        )
    }

    fn to_health_json(&self, now: DateTime<Utc>) -> Value {
        json!({
            "lifecycle_state": self.lifecycle_state,
            "runtime_status": self.runtime_status,
            "severity": self.severity,
            "reconnect_attempts": self.reconnect_attempts,
            "max_reconnect_attempts": self.max_reconnect_attempts,
            "next_reconnect_at": self.next_reconnect_at.map(|value| value.to_rfc3339()),
            "reconnect_due": self.next_reconnect_at.is_some_and(|deadline| deadline <= now),
            "last_error_code": self.last_error_code,
            "last_event_kind": self.last_event_kind,
            "updated_at": self.updated_at.to_rfc3339(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NativeMdRuntimeLifecycleEvent {
    provider_event_name: &'static str,
    event_kind: &'static str,
    runtime_status: &'static str,
    lifecycle_state: &'static str,
    severity: &'static str,
    metadata: Value,
    observed_at: DateTime<Utc>,
}

impl NativeMdRuntimeLifecycleEvent {
    fn new(
        provider_event_name: &'static str,
        event_kind: &'static str,
        runtime_status: &'static str,
        lifecycle_state: &'static str,
        severity: &'static str,
        metadata: Value,
        observed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            provider_event_name,
            event_kind,
            runtime_status,
            lifecycle_state,
            severity,
            metadata,
            observed_at,
        }
    }

    fn to_dto(&self, account_id: &str) -> WhatsAppSanitizedRuntimeEventDto {
        native_md_synthetic_runtime_lifecycle_dto(account_id, self)
    }
}

#[derive(Clone, Default)]
struct NativeMdRuntimeLifecycleRegistry {
    state: Arc<tokio::sync::Mutex<HashMap<String, NativeMdRuntimeLifecycleSnapshot>>>,
}

impl NativeMdRuntimeLifecycleRegistry {
    fn new() -> Self {
        Self::default()
    }

    async fn record_start_requested(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeActorStartRequested",
            "runtime.actor.start_requested",
            "degraded",
            "recovering",
            "info",
            None,
            None,
        )
        .await
    }

    async fn record_start_succeeded(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeActorStarted",
            "runtime.actor.started",
            "degraded",
            "recovering",
            "info",
            None,
            None,
        )
        .await
    }

    async fn record_stopped(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        let now = Utc::now();
        self.state.lock().await.insert(
            account_id.to_owned(),
            NativeMdRuntimeLifecycleSnapshot::stopped(now),
        );
        NativeMdRuntimeLifecycleEvent::new(
            "NativeMdRuntimeActorStopped",
            "runtime.actor.stopped",
            "stopped",
            "stopped",
            "info",
            native_md_lifecycle_metadata(
                "runtime.actor.stopped",
                "stopped",
                "stopped",
                "info",
                json!({
                    "reconnect_policy": "disabled_after_explicit_stop",
                }),
            ),
            now,
        )
    }

    async fn record_reconnect_started(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeReconnectStarted",
            "connection.reconnect.started",
            "degraded",
            "recovering",
            "info",
            None,
            Some(json!({
                "reconnect_policy": "tick_driven_restart_from_vault_session",
            })),
        )
        .await
    }

    async fn record_start_failed(
        &self,
        account_id: &str,
        error_code: &'static str,
    ) -> NativeMdRuntimeLifecycleEvent {
        self.record_degraded(
            account_id,
            "NativeMdRuntimeActorStartFailed",
            "runtime.actor.start_failed",
            Some(error_code.to_owned()),
            true,
        )
        .await
    }

    async fn record_reconnect_failed(
        &self,
        account_id: &str,
        error_code: &'static str,
    ) -> NativeMdRuntimeLifecycleEvent {
        self.record_degraded(
            account_id,
            "NativeMdRuntimeReconnectFailed",
            "connection.reconnect.failed",
            Some(error_code.to_owned()),
            true,
        )
        .await
    }

    async fn reconnect_due(&self, account_id: &str, now: DateTime<Utc>) -> bool {
        self.state
            .lock()
            .await
            .get(account_id)
            .and_then(|snapshot| snapshot.next_reconnect_at)
            .is_some_and(|deadline| deadline <= now)
    }

    async fn health(&self, account_id: &str) -> Value {
        let now = Utc::now();
        let snapshot = self
            .state
            .lock()
            .await
            .get(account_id)
            .cloned()
            .unwrap_or_else(|| NativeMdRuntimeLifecycleSnapshot::stopped(now));
        json!({
            "policy": "tick_driven_reconnect_from_vault_bound_session",
            "base_delay_seconds": NATIVE_MD_RECONNECT_BASE_DELAY_SECONDS,
            "max_delay_seconds": NATIVE_MD_RECONNECT_MAX_DELAY_SECONDS,
            "max_attempts": NATIVE_MD_RECONNECT_MAX_ATTEMPTS,
            "event_source": "sanitized_runtime_lifecycle_events",
            "completion_rule": "provider_observed_connected_event_marks_recovered",
            "direct_domain_calls": "forbidden",
            "state": snapshot.to_health_json(now),
        })
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn record_provider_event(
        &self,
        account_id: &str,
        event: &wa_rs::types::events::Event,
    ) -> Option<NativeMdRuntimeLifecycleEvent> {
        use wa_rs::types::events::Event;

        match event {
            Event::Connected(_) => Some(
                self.record_recovered(account_id, "provider.connected")
                    .await,
            ),
            Event::Disconnected(_) => Some(
                self.record_degraded(
                    account_id,
                    "NativeMdRuntimeConnectionDegraded",
                    "connection.degraded",
                    Some("provider_disconnected".to_owned()),
                    true,
                )
                .await,
            ),
            Event::StreamReplaced(_) => Some(
                self.record_degraded(
                    account_id,
                    "NativeMdRuntimeConnectionDegraded",
                    "connection.degraded",
                    Some("stream_replaced".to_owned()),
                    true,
                )
                .await,
            ),
            Event::StreamError(stream_error) => Some(
                self.record_degraded(
                    account_id,
                    "NativeMdRuntimeConnectionDegraded",
                    "connection.degraded",
                    Some(format!("stream_error_{}", stream_error.code)),
                    true,
                )
                .await,
            ),
            Event::ConnectFailure(connect_failure) => {
                let should_reconnect = connect_failure.reason.should_reconnect()
                    && !connect_failure.reason.is_logged_out();
                Some(
                    self.record_degraded(
                        account_id,
                        "NativeMdRuntimeConnectionDegraded",
                        "connection.degraded",
                        Some(format!("connect_failure_{}", connect_failure.reason.code())),
                        should_reconnect,
                    )
                    .await,
                )
            }
            Event::LoggedOut(logged_out) => Some(
                self.record_fixed_state(
                    account_id,
                    "NativeMdRuntimeConnectionRevoked",
                    "connection.revoked",
                    "revoked",
                    "revoked",
                    "warning",
                    Some(format!("logged_out_{}", logged_out.reason.code())),
                    Some(json!({
                        "reconnect_scheduled": false,
                        "requires_user_action": true,
                    })),
                )
                .await,
            ),
            Event::TemporaryBan(temporary_ban) => Some(
                self.record_fixed_state(
                    account_id,
                    "NativeMdRuntimeConnectionDegraded",
                    "connection.degraded",
                    "degraded",
                    "degraded",
                    "warning",
                    Some(format!("temporary_ban_{}", temporary_ban.code.code())),
                    Some(json!({
                        "reconnect_scheduled": false,
                        "temporary_ban": true,
                        "expires_in_seconds": temporary_ban.expire.num_seconds(),
                    })),
                )
                .await,
            ),
            _ => None,
        }
    }

    async fn record_recovered(
        &self,
        account_id: &str,
        recovery_source: &'static str,
    ) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeRecovered",
            "connection.recovered",
            "available",
            "available",
            "info",
            None,
            Some(json!({
                "recovery_source": recovery_source,
                "reconnect_attempts_reset": true,
            })),
        )
        .await
    }

    async fn record_degraded(
        &self,
        account_id: &str,
        provider_event_name: &'static str,
        event_kind: &'static str,
        error_code: Option<String>,
        should_reconnect: bool,
    ) -> NativeMdRuntimeLifecycleEvent {
        let now = Utc::now();
        let mut state = self.state.lock().await;
        let previous = state
            .get(account_id)
            .cloned()
            .unwrap_or_else(|| NativeMdRuntimeLifecycleSnapshot::stopped(now));
        let reconnect_attempts = if should_reconnect {
            previous.reconnect_attempts.saturating_add(1)
        } else {
            previous.reconnect_attempts
        };
        let reconnect_exhausted =
            should_reconnect && reconnect_attempts >= NATIVE_MD_RECONNECT_MAX_ATTEMPTS;
        let next_reconnect_at = if should_reconnect && !reconnect_exhausted {
            Some(now + native_md_reconnect_delay(reconnect_attempts))
        } else {
            None
        };
        let snapshot = NativeMdRuntimeLifecycleSnapshot::new(
            "degraded",
            "degraded",
            "warning",
            reconnect_attempts,
            next_reconnect_at,
            error_code.clone(),
            event_kind,
            now,
        );
        state.insert(account_id.to_owned(), snapshot);
        drop(state);

        NativeMdRuntimeLifecycleEvent::new(
            provider_event_name,
            event_kind,
            "degraded",
            "degraded",
            "warning",
            native_md_lifecycle_metadata(
                event_kind,
                "degraded",
                "degraded",
                "warning",
                json!({
                    "error_code": error_code,
                    "should_reconnect": should_reconnect,
                    "reconnect_scheduled": next_reconnect_at.is_some(),
                    "reconnect_exhausted": reconnect_exhausted,
                    "reconnect_attempts": reconnect_attempts,
                    "max_reconnect_attempts": NATIVE_MD_RECONNECT_MAX_ATTEMPTS,
                    "next_reconnect_at": next_reconnect_at.map(|value| value.to_rfc3339()),
                }),
            ),
            now,
        )
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_fixed_state(
        &self,
        account_id: &str,
        provider_event_name: &'static str,
        event_kind: &'static str,
        runtime_status: &'static str,
        lifecycle_state: &'static str,
        severity: &'static str,
        error_code: Option<String>,
        extra_metadata: Option<Value>,
    ) -> NativeMdRuntimeLifecycleEvent {
        let now = Utc::now();
        self.state.lock().await.insert(
            account_id.to_owned(),
            NativeMdRuntimeLifecycleSnapshot::new(
                lifecycle_state,
                runtime_status,
                severity,
                0,
                None,
                error_code.clone(),
                event_kind,
                now,
            ),
        );
        NativeMdRuntimeLifecycleEvent::new(
            provider_event_name,
            event_kind,
            runtime_status,
            lifecycle_state,
            severity,
            native_md_lifecycle_metadata(
                event_kind,
                runtime_status,
                lifecycle_state,
                severity,
                extra_metadata.unwrap_or_else(|| {
                    json!({
                        "error_code": error_code,
                    })
                }),
            ),
            now,
        )
    }
}

fn native_md_reconnect_delay(attempt: u32) -> chrono::Duration {
    let exponent = attempt.saturating_sub(1).min(8);
    let multiplier = 2_i64.pow(exponent);
    let seconds = (NATIVE_MD_RECONNECT_BASE_DELAY_SECONDS * multiplier)
        .min(NATIVE_MD_RECONNECT_MAX_DELAY_SECONDS);
    chrono::Duration::seconds(seconds)
}

fn native_md_lifecycle_metadata(
    event_kind: &'static str,
    runtime_status: &'static str,
    lifecycle_state: &'static str,
    severity: &'static str,
    extra_metadata: Value,
) -> Value {
    let mut metadata = json!({
        "provider_event_kind": event_kind,
        "runtime_event_kind": event_kind,
        "runtime_status": runtime_status,
        "lifecycle_state": lifecycle_state,
        "severity": severity,
        "payload_policy": "sanitized_metadata_only",
        "message_body": "excluded",
        "media_bytes": "excluded",
        "session_material": "excluded",
        "raw_provider_payload": "excluded",
        "reconnect_policy": "tick_driven_reconnect_from_vault_bound_session",
    });
    if let (Some(base), Value::Object(extra)) = (metadata.as_object_mut(), extra_metadata) {
        for (key, value) in extra {
            base.insert(key, value);
        }
    }
    metadata
}

fn native_md_synthetic_runtime_lifecycle_dto(
    account_id: &str,
    event: &NativeMdRuntimeLifecycleEvent,
) -> WhatsAppSanitizedRuntimeEventDto {
    let provider_event_id = format!(
        "wa-rs:lifecycle:{}:{}",
        event.event_kind,
        event.observed_at.timestamp_micros()
    );
    WhatsAppSanitizedRuntimeEventDto {
        account_id: account_id.to_owned(),
        provider_event_id: provider_event_id.clone(),
        provider_shape: "whatsapp_native_md",
        runtime_driver: "wa-rs",
        provider_event_name: event.provider_event_name,
        event_family: NativeMdProviderEventFamily::RuntimeLifecycle.as_str(),
        raw_record_kind: "whatsapp_web_runtime_event",
        raw_signal_event_kind: "signal.raw.whatsapp.runtime_event.observed",
        accepted_event_kind: "signal.accepted.whatsapp.runtime_event",
        source_fingerprint_seed: format!(
            "source_fingerprint:v5:whatsapp_web_runtime_event:{}:{}:{}",
            account_id, event.provider_event_name, provider_event_id
        ),
        bridge_dispatch: native_md_runtime_event_bridge_dispatch(),
        metadata: event.metadata.clone(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdProviderEventFamily {
    Authentication,
    RuntimeLifecycle,
    SyncLifecycle,
    Message,
    MessageUpdate,
    MessageDelete,
    Receipt,
    Reaction,
    Dialog,
    Participant,
    Presence,
    CallMetadata,
    Status,
    StatusView,
    StatusDelete,
    Media,
    MediaLifecycle,
    CommandReconciliation,
    Unsupported,
}

impl NativeMdProviderEventFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::RuntimeLifecycle => "runtime.lifecycle",
            Self::SyncLifecycle => "sync.lifecycle",
            Self::Message => "message",
            Self::MessageUpdate => "message.update",
            Self::MessageDelete => "message.delete",
            Self::Receipt => "receipt",
            Self::Reaction => "reaction",
            Self::Dialog => "dialog",
            Self::Participant => "participant",
            Self::Presence => "presence",
            Self::CallMetadata => "call.metadata",
            Self::Status => "status",
            Self::StatusView => "status.view",
            Self::StatusDelete => "status.delete",
            Self::Media => "media",
            Self::MediaLifecycle => "media.lifecycle",
            Self::CommandReconciliation => "command.reconciliation",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdProviderEventClassification {
    provider_event_name: &'static str,
    family: NativeMdProviderEventFamily,
    raw_record_kind: &'static str,
    accepted_event_kind: &'static str,
    unsupported_evidence: bool,
}

impl NativeMdProviderEventClassification {
    fn new(
        provider_event_name: &'static str,
        family: NativeMdProviderEventFamily,
        raw_record_kind: &'static str,
        accepted_event_kind: &'static str,
    ) -> Self {
        Self {
            provider_event_name,
            family,
            raw_record_kind,
            accepted_event_kind,
            unsupported_evidence: family == NativeMdProviderEventFamily::Unsupported,
        }
    }

    fn authentication(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Authentication,
            "whatsapp_web_runtime_event",
            "signal.accepted.whatsapp.runtime_event",
        )
    }

    fn runtime_lifecycle(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::RuntimeLifecycle,
            "whatsapp_web_runtime_event",
            "signal.accepted.whatsapp.runtime_event",
        )
    }

    fn sync_lifecycle(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::SyncLifecycle,
            "whatsapp_web_runtime_event",
            "signal.accepted.whatsapp.runtime_event",
        )
    }

    fn message(provider_event_name: &'static str, family: NativeMdProviderEventFamily) -> Self {
        let raw_record_kind = match family {
            NativeMdProviderEventFamily::Message => "whatsapp_web_message",
            NativeMdProviderEventFamily::MessageUpdate => "whatsapp_web_message_update",
            NativeMdProviderEventFamily::MessageDelete => "whatsapp_web_message_delete",
            NativeMdProviderEventFamily::Reaction => "whatsapp_web_reaction",
            NativeMdProviderEventFamily::Media => "whatsapp_web_media",
            NativeMdProviderEventFamily::CallMetadata => "whatsapp_web_call",
            _ => "whatsapp_web_runtime_event",
        };
        let accepted_event_kind = match family {
            NativeMdProviderEventFamily::Message => "signal.accepted.whatsapp.message",
            NativeMdProviderEventFamily::MessageUpdate => "signal.accepted.whatsapp.message_update",
            NativeMdProviderEventFamily::MessageDelete => "signal.accepted.whatsapp.message_delete",
            NativeMdProviderEventFamily::Reaction => "signal.accepted.whatsapp.reaction",
            NativeMdProviderEventFamily::Media => "signal.accepted.whatsapp.media",
            NativeMdProviderEventFamily::CallMetadata => "signal.accepted.whatsapp.call",
            _ => "signal.accepted.whatsapp.runtime_event",
        };
        Self::new(
            provider_event_name,
            family,
            raw_record_kind,
            accepted_event_kind,
        )
    }

    fn receipt(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Receipt,
            "whatsapp_web_receipt",
            "signal.accepted.whatsapp.receipt",
        )
    }

    fn dialog(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Dialog,
            "whatsapp_web_dialog",
            "signal.accepted.whatsapp.dialog",
        )
    }

    fn participant(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Participant,
            "whatsapp_web_participant",
            "signal.accepted.whatsapp.participant",
        )
    }

    fn presence(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Presence,
            "whatsapp_web_presence",
            "signal.accepted.whatsapp.presence",
        )
    }

    fn unsupported(provider_event_name: &'static str) -> Self {
        Self::new(
            provider_event_name,
            NativeMdProviderEventFamily::Unsupported,
            "whatsapp_web_runtime_event",
            "signal.accepted.whatsapp.runtime_event",
        )
    }

    fn assert_signal_hub_boundary(self) {
        debug_assert!(!self.provider_event_name.is_empty());
        debug_assert!(!self.family.as_str().is_empty());
        debug_assert!(self.raw_record_kind.starts_with("whatsapp_web_"));
        debug_assert!(
            self.accepted_event_kind
                .starts_with("signal.accepted.whatsapp.")
        );
        debug_assert_eq!(
            self.unsupported_evidence,
            self.family == NativeMdProviderEventFamily::Unsupported
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdRawEvidencePayloadPolicy {
    SanitizedMetadataOnly,
}

impl NativeMdRawEvidencePayloadPolicy {
    fn as_str(self) -> &'static str {
        match self {
            Self::SanitizedMetadataOnly => "sanitized_metadata_only",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdRawEvidenceEnvelope<'a> {
    account_id: &'a str,
    provider_event_id: &'a str,
    provider_shape: &'static str,
    runtime_driver: &'static str,
    provider_event_name: &'static str,
    family: NativeMdProviderEventFamily,
    raw_record_kind: &'static str,
    raw_signal_event_kind: &'static str,
    accepted_event_kind: &'static str,
    source_fingerprint_seed: String,
    payload_policy: NativeMdRawEvidencePayloadPolicy,
    secret_policy: &'static str,
    message_body_policy: &'static str,
    media_bytes_policy: &'static str,
    unsupported_evidence: bool,
}

impl<'a> NativeMdRawEvidenceEnvelope<'a> {
    fn from_classification(
        account_id: &'a str,
        provider_event_id: &'a str,
        classification: NativeMdProviderEventClassification,
    ) -> Self {
        Self {
            account_id,
            provider_event_id,
            provider_shape: "whatsapp_native_md",
            runtime_driver: "wa-rs",
            provider_event_name: classification.provider_event_name,
            family: classification.family,
            raw_record_kind: classification.raw_record_kind,
            raw_signal_event_kind: native_md_raw_signal_event_kind(classification.raw_record_kind),
            accepted_event_kind: classification.accepted_event_kind,
            source_fingerprint_seed: format!(
                "source_fingerprint:v5:{}:{}:{}:{}",
                classification.raw_record_kind,
                account_id,
                classification.provider_event_name,
                provider_event_id
            ),
            payload_policy: NativeMdRawEvidencePayloadPolicy::SanitizedMetadataOnly,
            secret_policy: "no_session_token_cookie_or_raw_secret",
            message_body_policy: "no_message_body_in_runtime_metadata",
            media_bytes_policy: "no_media_bytes_in_postgres_events_or_logs",
            unsupported_evidence: classification.unsupported_evidence,
        }
    }

    fn assert_append_only_hub_contract(&self) {
        debug_assert!(!self.account_id.trim().is_empty());
        debug_assert!(!self.provider_event_id.trim().is_empty());
        debug_assert_eq!(self.provider_shape, "whatsapp_native_md");
        debug_assert_eq!(self.runtime_driver, "wa-rs");
        debug_assert!(!self.provider_event_name.trim().is_empty());
        debug_assert!(!self.family.as_str().is_empty());
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
        debug_assert_eq!(self.payload_policy.as_str(), "sanitized_metadata_only");
        debug_assert_eq!(self.secret_policy, "no_session_token_cookie_or_raw_secret");
        debug_assert_eq!(
            self.message_body_policy,
            "no_message_body_in_runtime_metadata"
        );
        debug_assert_eq!(
            self.media_bytes_policy,
            "no_media_bytes_in_postgres_events_or_logs"
        );
        debug_assert_eq!(
            self.unsupported_evidence,
            self.family == NativeMdProviderEventFamily::Unsupported
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NativeMdSanitizedProviderEventDto<'a> {
    envelope: NativeMdRawEvidenceEnvelope<'a>,
    bridge_dispatch: WhatsAppRuntimeBridgeDispatch,
    metadata: Value,
}

impl<'a> NativeMdSanitizedProviderEventDto<'a> {
    fn from_envelope(envelope: NativeMdRawEvidenceEnvelope<'a>, metadata: Value) -> Self {
        Self {
            bridge_dispatch: native_md_runtime_bridge_dispatch_for_family(envelope.family),
            envelope,
            metadata,
        }
    }

    fn assert_sanitized_contract(&self) {
        self.envelope.assert_append_only_hub_contract();
        self.bridge_dispatch.assert_runtime_bridge_contract();
        debug_assert_eq!(
            self.metadata.get("payload_policy").and_then(Value::as_str),
            Some("sanitized_metadata_only")
        );
        debug_assert_eq!(
            self.metadata.get("message_body").and_then(Value::as_str),
            Some("excluded")
        );
        debug_assert_eq!(
            self.metadata.get("media_bytes").and_then(Value::as_str),
            Some("excluded")
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

#[cfg(feature = "whatsapp-native-md-runtime")]
async fn native_md_owned_sanitized_runtime_event_dto(
    account_id: &str,
    event: &wa_rs::types::events::Event,
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
) -> WhatsAppSanitizedRuntimeEventDto {
    let provider_event_id = native_md_wa_rs_provider_event_id(event);
    let media_ref_materializations =
        native_md_materialize_media_download_refs(account_id, event, secret_store, vault).await;
    let dto = native_md_wa_rs_sanitized_event_dto_with_media_materialization(
        account_id,
        &provider_event_id,
        event,
        Some(&media_ref_materializations),
    );
    let owned = WhatsAppSanitizedRuntimeEventDto {
        account_id: dto.envelope.account_id.to_owned(),
        provider_event_id: dto.envelope.provider_event_id.to_owned(),
        provider_shape: dto.envelope.provider_shape,
        runtime_driver: dto.envelope.runtime_driver,
        provider_event_name: dto.envelope.provider_event_name,
        event_family: dto.envelope.family.as_str(),
        raw_record_kind: dto.envelope.raw_record_kind,
        raw_signal_event_kind: dto.envelope.raw_signal_event_kind,
        accepted_event_kind: dto.envelope.accepted_event_kind,
        source_fingerprint_seed: dto.envelope.source_fingerprint_seed,
        bridge_dispatch: dto.bridge_dispatch,
        metadata: dto.metadata,
    };
    owned.assert_event_spine_contract();
    owned
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Default)]
struct NativeMdSanitizedEventCaptureSink {
    accepted_count: Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdSanitizedEventCaptureSink {
    fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl WhatsAppRuntimeEventSink for NativeMdSanitizedEventCaptureSink {
    fn accept<'a>(
        &'a self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> super::WhatsAppRuntimeEventSinkFuture<'a> {
        Box::pin(async move {
            dto.assert_event_spine_contract();
            self.accepted_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
    }
}

fn native_md_runtime_bridge_dispatch_for_family(
    family: NativeMdProviderEventFamily,
) -> WhatsAppRuntimeBridgeDispatch {
    match family {
        NativeMdProviderEventFamily::Authentication
        | NativeMdProviderEventFamily::RuntimeLifecycle
        | NativeMdProviderEventFamily::Unsupported => native_md_runtime_event_bridge_dispatch(),
        NativeMdProviderEventFamily::SyncLifecycle => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            "WhatsAppRuntimeBridgeSyncLifecycleRequest",
            "provider_observed.runtime_bridge_sync_lifecycle",
        ),
        NativeMdProviderEventFamily::Message => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            "NewWhatsappWebMessage",
            "provider_observed.runtime_bridge_message",
        ),
        NativeMdProviderEventFamily::MessageUpdate => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
            "NewWhatsappWebMessageUpdate",
            "provider_observed.runtime_bridge_message_update",
        ),
        NativeMdProviderEventFamily::MessageDelete => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
            "NewWhatsappWebMessageDelete",
            "provider_observed.runtime_bridge_message_delete",
        ),
        NativeMdProviderEventFamily::Receipt => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
            "NewWhatsappWebReceipt",
            "provider_observed.runtime_bridge_receipt",
        ),
        NativeMdProviderEventFamily::Reaction => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
            "NewWhatsappWebReaction",
            "provider_observed.runtime_bridge_reaction",
        ),
        NativeMdProviderEventFamily::Dialog => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
            "NewWhatsappWebDialog",
            "provider_observed.runtime_bridge_dialog",
        ),
        NativeMdProviderEventFamily::Participant => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/participants",
            "NewWhatsappWebParticipant",
            "provider_observed.runtime_bridge_participant",
        ),
        NativeMdProviderEventFamily::Presence => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/presence",
            "NewWhatsappWebPresence",
            "provider_observed.runtime_bridge_presence",
        ),
        NativeMdProviderEventFamily::CallMetadata => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/calls",
            "NewWhatsappWebCall",
            "provider_observed.runtime_bridge_call",
        ),
        NativeMdProviderEventFamily::Status => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
            "NewWhatsappWebStatus",
            "provider_observed.runtime_bridge_status",
        ),
        NativeMdProviderEventFamily::StatusView => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
            "NewWhatsappWebStatusView",
            "provider_observed.runtime_bridge_status_view",
        ),
        NativeMdProviderEventFamily::StatusDelete => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
            "NewWhatsappWebStatusDelete",
            "provider_observed.runtime_bridge_status_delete",
        ),
        NativeMdProviderEventFamily::Media => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/media",
            "NewWhatsappWebMedia",
            "provider_observed.runtime_bridge_media",
        ),
        NativeMdProviderEventFamily::MediaLifecycle => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
            "WhatsAppRuntimeBridgeMediaLifecycleRequest",
            "provider_observed.runtime_bridge_media_lifecycle",
        ),
        NativeMdProviderEventFamily::CommandReconciliation => WhatsAppRuntimeBridgeDispatch::new(
            "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            "NewWhatsappWebRuntimeEvent",
            "provider_observed.runtime_bridge_command_reconciliation",
        ),
    }
}

fn native_md_runtime_event_bridge_dispatch() -> WhatsAppRuntimeBridgeDispatch {
    WhatsAppRuntimeBridgeDispatch::new(
        "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
        "NewWhatsappWebRuntimeEvent",
        "provider_observed.runtime_bridge_runtime_event",
    )
}

fn native_md_raw_signal_event_kind(raw_record_kind: &str) -> &'static str {
    match raw_record_kind {
        "whatsapp_web_reaction" => "signal.raw.whatsapp.reaction.observed",
        "whatsapp_web_media" => "signal.raw.whatsapp.media.observed",
        "whatsapp_web_status" => "signal.raw.whatsapp.status.observed",
        "whatsapp_web_status_view" => "signal.raw.whatsapp.status_view.observed",
        "whatsapp_web_status_delete" => "signal.raw.whatsapp.status_delete.observed",
        "whatsapp_web_presence" => "signal.raw.whatsapp.presence.observed",
        "whatsapp_web_call" => "signal.raw.whatsapp.call_metadata.observed",
        "whatsapp_web_runtime_event" => "signal.raw.whatsapp.runtime_event.observed",
        "whatsapp_web_dialog" => "signal.raw.whatsapp.dialog.observed",
        "whatsapp_web_participant" => "signal.raw.whatsapp.participant.observed",
        "whatsapp_web_message_update" => "signal.raw.whatsapp.message_update.observed",
        "whatsapp_web_message_delete" => "signal.raw.whatsapp.message_delete.observed",
        "whatsapp_web_receipt" => "signal.raw.whatsapp.receipt.observed",
        _ => "signal.raw.whatsapp.message.observed",
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_raw_evidence_envelope<'a>(
    account_id: &'a str,
    provider_event_id: &'a str,
    event: &wa_rs::types::events::Event,
) -> NativeMdRawEvidenceEnvelope<'a> {
    NativeMdRawEvidenceEnvelope::from_classification(
        account_id,
        provider_event_id,
        classify_wa_rs_event(event),
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_event_dto<'a>(
    account_id: &'a str,
    provider_event_id: &'a str,
    event: &wa_rs::types::events::Event,
) -> NativeMdSanitizedProviderEventDto<'a> {
    native_md_wa_rs_sanitized_event_dto_with_media_materialization(
        account_id,
        provider_event_id,
        event,
        None,
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_event_dto_with_media_materialization<'a>(
    account_id: &'a str,
    provider_event_id: &'a str,
    event: &wa_rs::types::events::Event,
    media_ref_materializations: Option<&BTreeMap<String, NativeMdMediaDownloadRefMaterialization>>,
) -> NativeMdSanitizedProviderEventDto<'a> {
    let envelope = native_md_wa_rs_raw_evidence_envelope(account_id, provider_event_id, event);
    let metadata = native_md_wa_rs_sanitized_metadata_with_media_materialization(
        event,
        media_ref_materializations,
    );
    let dto = NativeMdSanitizedProviderEventDto::from_envelope(envelope, metadata);
    dto.assert_sanitized_contract();
    dto
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_provider_event_id(event: &wa_rs::types::events::Event) -> String {
    use sha2::Digest as _;

    let metadata = native_md_wa_rs_sanitized_metadata(event);
    let metadata_bytes = serde_json::to_vec(&metadata).unwrap_or_default();
    let mut hasher = sha2::Sha256::new();
    hasher.update(native_md_wa_rs_event_name(event).as_bytes());
    hasher.update([0]);
    hasher.update(metadata_bytes);
    let digest = hasher.finalize();
    format!(
        "wa-rs:{}:sha256:{}",
        native_md_wa_rs_event_name(event),
        native_md_hex(&digest)
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn classify_wa_rs_event(
    event: &wa_rs::types::events::Event,
) -> NativeMdProviderEventClassification {
    use wa_rs::types::events::Event;

    match event {
        Event::PairSuccess(_)
        | Event::PairError(_)
        | Event::PairingQrCode { .. }
        | Event::PairingCode { .. }
        | Event::QrScannedWithoutMultidevice(_)
        | Event::ClientOutdated(_)
        | Event::LoggedOut(_) => {
            NativeMdProviderEventClassification::authentication(native_md_wa_rs_event_name(event))
        }
        Event::Connected(_)
        | Event::Disconnected(_)
        | Event::StreamReplaced(_)
        | Event::TemporaryBan(_)
        | Event::ConnectFailure(_)
        | Event::StreamError(_) => NativeMdProviderEventClassification::runtime_lifecycle(
            native_md_wa_rs_event_name(event),
        ),
        Event::HistorySync(_) | Event::OfflineSyncPreview(_) | Event::OfflineSyncCompleted(_) => {
            NativeMdProviderEventClassification::sync_lifecycle(native_md_wa_rs_event_name(event))
        }
        Event::Message(message, info) => NativeMdProviderEventClassification::message(
            native_md_wa_rs_event_name(event),
            classify_wa_rs_message_event(message, info),
        ),
        Event::Receipt(_) => {
            NativeMdProviderEventClassification::receipt(native_md_wa_rs_event_name(event))
        }
        Event::ChatPresence(_) | Event::Presence(_) => {
            NativeMdProviderEventClassification::presence(native_md_wa_rs_event_name(event))
        }
        Event::JoinedGroup(_)
        | Event::GroupInfoUpdate { .. }
        | Event::PinUpdate(_)
        | Event::MuteUpdate(_)
        | Event::ArchiveUpdate(_)
        | Event::MarkChatAsReadUpdate(_) => {
            NativeMdProviderEventClassification::dialog(native_md_wa_rs_event_name(event))
        }
        Event::PictureUpdate(_)
        | Event::UserAboutUpdate(_)
        | Event::ContactUpdate(_)
        | Event::PushNameUpdate(_)
        | Event::SelfPushNameUpdated(_)
        | Event::DeviceListUpdate(_) => {
            NativeMdProviderEventClassification::participant(native_md_wa_rs_event_name(event))
        }
        Event::Notification(_) | Event::BusinessStatusUpdate(_) => {
            NativeMdProviderEventClassification::unsupported(native_md_wa_rs_event_name(event))
        }
        Event::UndecryptableMessage(_) => NativeMdProviderEventClassification::message(
            native_md_wa_rs_event_name(event),
            NativeMdProviderEventFamily::Message,
        ),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_metadata(event: &wa_rs::types::events::Event) -> Value {
    native_md_wa_rs_sanitized_metadata_with_media_materialization(event, None)
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_metadata_with_media_materialization(
    event: &wa_rs::types::events::Event,
    media_ref_materializations: Option<&BTreeMap<String, NativeMdMediaDownloadRefMaterialization>>,
) -> Value {
    let classification = classify_wa_rs_event(event);
    json!({
        "provider_event_name": classification.provider_event_name,
        "family": classification.family.as_str(),
        "raw_record_kind": classification.raw_record_kind,
        "accepted_event_kind": classification.accepted_event_kind,
        "runtime_bridge": {
            "endpoint_path": native_md_runtime_bridge_dispatch_for_family(classification.family).endpoint_path,
            "request_kind": native_md_runtime_bridge_dispatch_for_family(classification.family).request_kind,
            "observed_source": native_md_runtime_bridge_dispatch_for_family(classification.family).observed_source,
        },
        "payload_policy": NativeMdRawEvidencePayloadPolicy::SanitizedMetadataOnly.as_str(),
        "message_body": "excluded",
        "media_bytes": "excluded",
        "session_material": "excluded",
        "raw_provider_payload": "excluded",
        "redaction": {
            "qr_code": "excluded",
            "pair_code": "excluded",
            "raw_node": "excluded",
            "protobuf_action": "excluded",
            "history_sync_payload": "excluded",
            "about_text": "excluded",
            "push_names": "excluded",
            "business_profile_payload": "excluded"
        },
        "provider": native_md_wa_rs_sanitized_event_details(
            event,
            media_ref_materializations,
        ),
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_event_details(
    event: &wa_rs::types::events::Event,
    media_ref_materializations: Option<&BTreeMap<String, NativeMdMediaDownloadRefMaterialization>>,
) -> Value {
    use wa_rs::types::events::Event;

    match event {
        Event::Connected(_) => json!({
            "connection_state": "connected",
        }),
        Event::Disconnected(_) => json!({
            "connection_state": "disconnected",
        }),
        Event::PairSuccess(pair_success) => json!({
            "account_jid": native_md_jid_string(&pair_success.id),
            "lid_jid": native_md_jid_string(&pair_success.lid),
            "business_name_present": !pair_success.business_name.trim().is_empty(),
            "platform": pair_success.platform,
        }),
        Event::PairError(pair_error) => json!({
            "account_jid": native_md_jid_string(&pair_error.id),
            "lid_jid": native_md_jid_string(&pair_error.lid),
            "business_name_present": !pair_error.business_name.trim().is_empty(),
            "platform": pair_error.platform,
            "error": pair_error.error,
        }),
        Event::LoggedOut(logged_out) => json!({
            "on_connect": logged_out.on_connect,
            "reason": format!("{:?}", logged_out.reason),
            "reason_code": logged_out.reason.code(),
        }),
        Event::PairingQrCode { timeout, .. } => json!({
            "timeout_seconds": timeout.as_secs(),
            "qr_code": "excluded",
        }),
        Event::PairingCode { timeout, .. } => json!({
            "timeout_seconds": timeout.as_secs(),
            "pair_code": "excluded",
        }),
        Event::QrScannedWithoutMultidevice(_) => json!({
            "qr_scanned_without_multidevice": true,
        }),
        Event::ClientOutdated(_) => json!({
            "client_outdated": true,
        }),
        Event::StreamReplaced(_) => json!({
            "connection_state": "stream_replaced",
        }),
        Event::TemporaryBan(temporary_ban) => json!({
            "ban_code": temporary_ban.code.code(),
            "ban_reason": format!("{:?}", temporary_ban.code),
            "expires_in_seconds": temporary_ban.expire.num_seconds(),
        }),
        Event::ConnectFailure(connect_failure) => json!({
            "reason": format!("{:?}", connect_failure.reason),
            "reason_code": connect_failure.reason.code(),
            "should_reconnect": connect_failure.reason.should_reconnect(),
            "logged_out": connect_failure.reason.is_logged_out(),
            "message_present": !connect_failure.message.trim().is_empty(),
            "raw_node": "excluded",
        }),
        Event::StreamError(stream_error) => json!({
            "code": stream_error.code,
            "raw_node": "excluded",
        }),
        Event::HistorySync(_) => json!({
            "history_sync_payload": "excluded",
            "history_sync_present": true,
        }),
        Event::OfflineSyncPreview(preview) => json!({
            "total": preview.total,
            "app_data_changes": preview.app_data_changes,
            "messages": preview.messages,
            "notifications": preview.notifications,
            "receipts": preview.receipts,
        }),
        Event::OfflineSyncCompleted(completed) => json!({
            "count": completed.count,
        }),
        Event::Message(message, info) => {
            native_md_wa_rs_sanitized_message_metadata(message, info, media_ref_materializations)
        }
        Event::Receipt(receipt) => json!({
            "source": native_md_wa_rs_sanitized_message_source(&receipt.source),
            "message_ids": receipt.message_ids,
            "message_count": receipt.message_ids.len(),
            "message_sender_jid": native_md_jid_string(&receipt.message_sender),
            "receipt_type": format!("{:?}", receipt.r#type),
            "timestamp": receipt.timestamp.to_rfc3339(),
        }),
        Event::UndecryptableMessage(undecryptable) => json!({
            "message": native_md_wa_rs_sanitized_message_info(&undecryptable.info),
            "is_unavailable": undecryptable.is_unavailable,
            "unavailable_type": format!("{:?}", undecryptable.unavailable_type),
            "decrypt_fail_mode": format!("{:?}", undecryptable.decrypt_fail_mode),
            "message_body": "excluded",
        }),
        Event::Notification(_) => json!({
            "unsupported_raw_notification": true,
            "raw_node": "excluded",
        }),
        Event::ChatPresence(presence) => json!({
            "source": native_md_wa_rs_sanitized_message_source(&presence.source),
            "state": format!("{:?}", presence.state),
            "media": format!("{:?}", presence.media),
        }),
        Event::Presence(presence) => json!({
            "from_jid": native_md_jid_string(&presence.from),
            "unavailable": presence.unavailable,
            "last_seen": presence.last_seen.map(|timestamp| timestamp.to_rfc3339()),
        }),
        Event::PictureUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "author_jid": native_md_jid_string(&update.author),
            "timestamp": update.timestamp.to_rfc3339(),
            "photo_change_present": update.photo_change.is_some(),
        }),
        Event::UserAboutUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "about_text": "excluded",
            "about_text_present": !update.status.trim().is_empty(),
        }),
        Event::JoinedGroup(conversation) => json!({
            "conversation_id": conversation.get().map(|conversation| conversation.id.clone()),
            "conversation_payload": "excluded",
        }),
        Event::GroupInfoUpdate { jid, .. } => json!({
            "jid": native_md_jid_string(jid),
            "protobuf_action": "excluded",
        }),
        Event::ContactUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "from_full_sync": update.from_full_sync,
            "protobuf_action": "excluded",
        }),
        Event::PushNameUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "message": native_md_wa_rs_sanitized_message_info(&update.message),
            "old_push_name_present": !update.old_push_name.trim().is_empty(),
            "new_push_name_present": !update.new_push_name.trim().is_empty(),
            "push_names": "excluded",
        }),
        Event::SelfPushNameUpdated(update) => json!({
            "from_server": update.from_server,
            "old_name_present": !update.old_name.trim().is_empty(),
            "new_name_present": !update.new_name.trim().is_empty(),
            "push_names": "excluded",
        }),
        Event::PinUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "from_full_sync": update.from_full_sync,
            "protobuf_action": "excluded",
        }),
        Event::MuteUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "from_full_sync": update.from_full_sync,
            "protobuf_action": "excluded",
        }),
        Event::ArchiveUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "from_full_sync": update.from_full_sync,
            "protobuf_action": "excluded",
        }),
        Event::MarkChatAsReadUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "timestamp": update.timestamp.to_rfc3339(),
            "from_full_sync": update.from_full_sync,
            "protobuf_action": "excluded",
        }),
        Event::DeviceListUpdate(update) => json!({
            "user_jid": native_md_jid_string(&update.user),
            "lid_user_jid": update.lid_user.as_ref().map(native_md_jid_string),
            "update_type": format!("{:?}", update.update_type),
            "device_count": update.devices.len(),
            "key_index_present": update.key_index.is_some(),
            "contact_hash_present": update.contact_hash.is_some(),
        }),
        Event::BusinessStatusUpdate(update) => json!({
            "jid": native_md_jid_string(&update.jid),
            "target_jid": update.target_jid.as_ref().map(native_md_jid_string),
            "update_type": format!("{:?}", update.update_type),
            "timestamp": update.timestamp,
            "hash_present": update.hash.is_some(),
            "verified_name_present": update.verified_name.is_some(),
            "product_count": update.product_ids.len(),
            "collection_count": update.collection_ids.len(),
            "subscription_count": update.subscriptions.len(),
            "business_profile_payload": "excluded",
        }),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_message_metadata(
    message: &wa_rs::wa_rs_proto::whatsapp::Message,
    info: &wa_rs::types::message::MessageInfo,
    media_ref_materializations: Option<&BTreeMap<String, NativeMdMediaDownloadRefMaterialization>>,
) -> Value {
    let media_download_refs =
        native_md_wa_rs_sanitized_media_download_refs(message, media_ref_materializations);
    json!({
        "info": native_md_wa_rs_sanitized_message_info(info),
        "payload_shape": {
            "has_media": message_has_media_payload(message),
            "has_direct_media_download_ref": !media_download_refs.is_empty(),
            "has_nested_media_payload": message_has_nested_media_payload(message),
            "has_reaction": message.reaction_message.is_some() || message.enc_reaction_message.is_some(),
            "has_call": message.call.is_some() || message.scheduled_call_creation_message.is_some(),
            "has_edit": message.edited_message.is_some(),
            "protocol_message_type": message.protocol_message.as_ref().and_then(|protocol| protocol.r#type),
        },
        "media_download_refs": {
            "ref_count": media_download_refs.len(),
            "refs": media_download_refs,
            "secret_purpose": "whatsapp_media_download_ref",
            "materialization": "host_vault_required_before_live_download",
            "database_policy": "metadata_hashes_only_no_media_key_direct_path_or_url",
            "event_policy": "sanitized_metadata_only",
            "media_key": "excluded",
            "direct_path": "excluded",
            "static_url": "excluded",
            "bytes": "excluded",
        },
        "message_body": "excluded",
        "media_bytes": "excluded",
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Debug)]
struct NativeMdMediaDownloadRefRaw<'a> {
    media_type: &'static str,
    content_type: Option<&'a str>,
    file_length: Option<u64>,
    file_sha256: Option<&'a [u8]>,
    file_enc_sha256: Option<&'a [u8]>,
    media_key: Option<&'a [u8]>,
    direct_path: Option<&'a str>,
    static_url: Option<&'a str>,
    media_key_timestamp: Option<i64>,
    safe_details: Value,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdMediaDownloadRefRaw<'_> {
    fn fingerprint_seed(&self) -> String {
        native_md_media_ref_fingerprint_seed(
            self.media_type,
            self.file_sha256,
            self.file_enc_sha256,
            self.direct_path,
            self.static_url,
        )
    }

    fn fingerprint(&self) -> String {
        native_md_sha256_hex(self.fingerprint_seed().as_bytes())
    }

    fn has_provider_download_ref(&self) -> bool {
        self.direct_path.is_some() || self.static_url.is_some() || self.media_key.is_some()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_raw_media_download_refs(
    message: &wa_rs::wa_rs_proto::whatsapp::Message,
) -> Vec<NativeMdMediaDownloadRefRaw<'_>> {
    let mut refs = Vec::new();

    if let Some(image) = message.image_message.as_ref() {
        refs.push(NativeMdMediaDownloadRefRaw {
            media_type: "image",
            content_type: image.mimetype.as_deref(),
            file_length: image.file_length,
            file_sha256: image.file_sha256.as_deref(),
            file_enc_sha256: image.file_enc_sha256.as_deref(),
            media_key: image.media_key.as_deref(),
            direct_path: image.direct_path.as_deref(),
            static_url: image.static_url.as_deref(),
            media_key_timestamp: image.media_key_timestamp,
            safe_details: json!({
                "caption": "excluded",
                "caption_present": image.caption.as_ref().is_some_and(|value| !value.trim().is_empty()),
                "width": image.width,
                "height": image.height,
            }),
        });
    }
    if let Some(video) = message.video_message.as_ref() {
        refs.push(NativeMdMediaDownloadRefRaw {
            media_type: "video",
            content_type: video.mimetype.as_deref(),
            file_length: video.file_length,
            file_sha256: video.file_sha256.as_deref(),
            file_enc_sha256: video.file_enc_sha256.as_deref(),
            media_key: video.media_key.as_deref(),
            direct_path: video.direct_path.as_deref(),
            static_url: video.static_url.as_deref(),
            media_key_timestamp: video.media_key_timestamp,
            safe_details: json!({
                "caption": "excluded",
                "caption_present": video.caption.as_ref().is_some_and(|value| !value.trim().is_empty()),
                "seconds": video.seconds,
                "gif_playback": video.gif_playback,
                "width": video.width,
                "height": video.height,
            }),
        });
    }
    if let Some(audio) = message.audio_message.as_ref() {
        refs.push(NativeMdMediaDownloadRefRaw {
            media_type: "audio",
            content_type: audio.mimetype.as_deref(),
            file_length: audio.file_length,
            file_sha256: audio.file_sha256.as_deref(),
            file_enc_sha256: audio.file_enc_sha256.as_deref(),
            media_key: audio.media_key.as_deref(),
            direct_path: audio.direct_path.as_deref(),
            static_url: None,
            media_key_timestamp: audio.media_key_timestamp,
            safe_details: json!({
                "seconds": audio.seconds,
                "voice_note": audio.ptt.unwrap_or(false),
            }),
        });
    }
    if let Some(document) = message.document_message.as_ref() {
        refs.push(NativeMdMediaDownloadRefRaw {
            media_type: "document",
            content_type: document.mimetype.as_deref(),
            file_length: document.file_length,
            file_sha256: document.file_sha256.as_deref(),
            file_enc_sha256: document.file_enc_sha256.as_deref(),
            media_key: document.media_key.as_deref(),
            direct_path: document.direct_path.as_deref(),
            static_url: None,
            media_key_timestamp: document.media_key_timestamp,
            safe_details: json!({
                "filename": "excluded",
                "filename_present": document.file_name.as_ref().is_some_and(|value| !value.trim().is_empty()),
                "title": "excluded",
                "title_present": document.title.as_ref().is_some_and(|value| !value.trim().is_empty()),
                "page_count": document.page_count,
                "contact_vcard": document.contact_vcard.unwrap_or(false),
            }),
        });
    }
    if let Some(sticker) = message.sticker_message.as_ref() {
        refs.push(NativeMdMediaDownloadRefRaw {
            media_type: "sticker",
            content_type: sticker.mimetype.as_deref(),
            file_length: sticker.file_length,
            file_sha256: sticker.file_sha256.as_deref(),
            file_enc_sha256: sticker.file_enc_sha256.as_deref(),
            media_key: sticker.media_key.as_deref(),
            direct_path: sticker.direct_path.as_deref(),
            static_url: None,
            media_key_timestamp: sticker.media_key_timestamp,
            safe_details: json!({
                "width": sticker.width,
                "height": sticker.height,
                "first_frame_length": sticker.first_frame_length,
            }),
        });
    }

    refs
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_media_download_refs(
    message: &wa_rs::wa_rs_proto::whatsapp::Message,
    media_ref_materializations: Option<&BTreeMap<String, NativeMdMediaDownloadRefMaterialization>>,
) -> Vec<Value> {
    native_md_wa_rs_raw_media_download_refs(message)
        .into_iter()
        .map(|raw_ref| {
            let fingerprint = raw_ref.fingerprint();
            let materialization =
                media_ref_materializations.and_then(|entries| entries.get(&fingerprint));
            native_md_wa_rs_media_ref_metadata(&raw_ref, materialization)
        })
        .collect()
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_media_ref_metadata(
    raw_ref: &NativeMdMediaDownloadRefRaw<'_>,
    materialization: Option<&NativeMdMediaDownloadRefMaterialization>,
) -> Value {
    let fingerprint = raw_ref.fingerprint();
    json!({
        "media_type": raw_ref.media_type,
        "content_type": raw_ref.content_type,
        "file_length": raw_ref.file_length,
        "file_sha256": raw_ref.file_sha256.map(native_md_digest_hex),
        "file_enc_sha256": raw_ref.file_enc_sha256.map(native_md_digest_hex),
        "direct_path": "excluded",
        "direct_path_sha256": raw_ref.direct_path.map(|value| native_md_sha256_hex(value.as_bytes())),
        "static_url": "excluded",
        "static_url_sha256": raw_ref.static_url.map(|value| native_md_sha256_hex(value.as_bytes())),
        "media_key": "excluded",
        "media_key_sha256": raw_ref.media_key.map(native_md_sha256_hex),
        "media_key_present": raw_ref.media_key.is_some(),
        "media_key_timestamp": raw_ref.media_key_timestamp,
        "provider_media_ref_fingerprint": fingerprint,
        "host_vault_secret_purpose": "whatsapp_media_download_ref",
        "host_vault_secret_ref": materialization.map(|entry| entry.secret_ref.as_str()),
        "host_vault_materialization": {
            "status": materialization
                .map(|entry| entry.status.as_str())
                .unwrap_or("not_attempted_without_live_vault_context"),
            "secret_ref": materialization.map(|entry| entry.secret_ref.as_str()),
            "error_code": materialization.and_then(|entry| entry.error_code.as_deref()),
            "payload_policy": "secret_ref_only_raw_refs_excluded",
        },
        "downloadable_contract": {
            "requires_direct_path_or_static_url": true,
            "direct_path_present": raw_ref.direct_path.is_some(),
            "static_url_present": raw_ref.static_url.is_some(),
            "requires_media_key_for_encrypted_media": raw_ref.media_key.is_some(),
            "requires_file_sha256": raw_ref.file_sha256.is_some(),
            "requires_file_enc_sha256_for_encrypted_media": raw_ref.media_key.is_some(),
        },
        "payload_policy": "metadata_hashes_only_no_media_key_direct_path_url_or_bytes",
        "details": raw_ref.safe_details.clone(),
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_ref_fingerprint_seed(
    media_type: &str,
    file_sha256: Option<&[u8]>,
    file_enc_sha256: Option<&[u8]>,
    direct_path: Option<&str>,
    static_url: Option<&str>,
) -> String {
    format!(
        "native-md-media-ref:v1:{media_type}:{}:{}:{}:{}",
        file_sha256.map(native_md_digest_hex).unwrap_or_default(),
        file_enc_sha256
            .map(native_md_digest_hex)
            .unwrap_or_default(),
        direct_path
            .map(|value| native_md_sha256_hex(value.as_bytes()))
            .unwrap_or_default(),
        static_url
            .map(|value| native_md_sha256_hex(value.as_bytes()))
            .unwrap_or_default(),
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdMediaDownloadRefMaterialization {
    secret_ref: String,
    status: String,
    error_code: Option<String>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdMediaDownloadRefMaterialization {
    fn stored(secret_ref: String) -> Self {
        Self {
            secret_ref,
            status: "stored".to_owned(),
            error_code: None,
        }
    }

    fn skipped(secret_ref: String, error_code: &'static str) -> Self {
        Self {
            secret_ref,
            status: "skipped".to_owned(),
            error_code: Some(error_code.to_owned()),
        }
    }

    fn failed(secret_ref: String, error_code: &'static str) -> Self {
        Self {
            secret_ref,
            status: "failed".to_owned(),
            error_code: Some(error_code.to_owned()),
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
async fn native_md_materialize_media_download_refs(
    account_id: &str,
    event: &wa_rs::types::events::Event,
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
) -> BTreeMap<String, NativeMdMediaDownloadRefMaterialization> {
    use wa_rs::types::events::Event;

    let Event::Message(message, _) = event else {
        return BTreeMap::new();
    };

    let mut materializations = BTreeMap::new();
    for raw_ref in native_md_wa_rs_raw_media_download_refs(message) {
        let fingerprint = raw_ref.fingerprint();
        let secret_ref = native_md_media_download_secret_ref(account_id, &fingerprint);
        let materialization = if raw_ref.has_provider_download_ref() {
            native_md_store_media_download_ref(
                account_id,
                &secret_ref,
                &fingerprint,
                &raw_ref,
                secret_store,
                vault,
            )
            .await
        } else {
            NativeMdMediaDownloadRefMaterialization::skipped(
                secret_ref,
                "native_md_media_ref_missing_provider_download_ref",
            )
        };
        materializations.insert(fingerprint, materialization);
    }
    materializations
}

#[cfg(feature = "whatsapp-native-md-runtime")]
async fn native_md_store_media_download_ref(
    account_id: &str,
    secret_ref: &str,
    fingerprint: &str,
    raw_ref: &NativeMdMediaDownloadRefRaw<'_>,
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
) -> NativeMdMediaDownloadRefMaterialization {
    let metadata = native_md_media_download_secret_metadata(account_id, fingerprint, raw_ref);
    if secret_store
        .upsert_secret_reference(
            &NewSecretReference::new(
                secret_ref,
                SecretKind::Other,
                SecretStoreKind::HostVault,
                "WhatsApp media download ref",
            )
            .metadata(metadata.clone()),
        )
        .await
        .is_err()
    {
        return NativeMdMediaDownloadRefMaterialization::failed(
            secret_ref.to_owned(),
            "native_md_media_ref_secret_reference_upsert_failed",
        );
    }

    let payload = match serde_json::to_string(&native_md_media_download_secret_payload(
        account_id,
        fingerprint,
        raw_ref,
    )) {
        Ok(payload) => payload,
        Err(_) => {
            return NativeMdMediaDownloadRefMaterialization::failed(
                secret_ref.to_owned(),
                "native_md_media_ref_secret_payload_serialization_failed",
            );
        }
    };

    match vault.store_secret(
        secret_ref,
        &payload,
        crate::vault::SecretEntryContext {
            entry_kind: "provider_media_download_ref",
            account_id,
            purpose: "whatsapp_media_download_ref",
            secret_kind: SecretKind::Other.as_str(),
            label: "WhatsApp media download ref",
            metadata: &metadata,
        },
    ) {
        Ok(()) => NativeMdMediaDownloadRefMaterialization::stored(secret_ref.to_owned()),
        Err(_) => NativeMdMediaDownloadRefMaterialization::failed(
            secret_ref.to_owned(),
            "native_md_media_ref_host_vault_store_failed",
        ),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_download_secret_ref(account_id: &str, fingerprint: &str) -> String {
    let suffix = fingerprint
        .strip_prefix("sha256:")
        .unwrap_or(fingerprint)
        .chars()
        .take(32)
        .collect::<String>();
    format!(
        "secret:provider-account:{}:whatsapp_media_download_ref:{}",
        account_id.trim(),
        suffix
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_download_secret_metadata(
    account_id: &str,
    fingerprint: &str,
    raw_ref: &NativeMdMediaDownloadRefRaw<'_>,
) -> Value {
    json!({
        "provider": "whatsapp",
        "provider_shape": "whatsapp_native_md",
        "runtime_driver": "wa-rs",
        "account_id": account_id,
        "secret_purpose": "whatsapp_media_download_ref",
        "provider_media_ref_fingerprint": fingerprint,
        "media_type": raw_ref.media_type,
        "content_type": raw_ref.content_type,
        "file_length": raw_ref.file_length,
        "file_sha256": raw_ref.file_sha256.map(native_md_digest_hex),
        "file_enc_sha256": raw_ref.file_enc_sha256.map(native_md_digest_hex),
        "direct_path": "excluded",
        "direct_path_sha256": raw_ref.direct_path.map(|value| native_md_sha256_hex(value.as_bytes())),
        "static_url": "excluded",
        "static_url_sha256": raw_ref.static_url.map(|value| native_md_sha256_hex(value.as_bytes())),
        "media_key": "excluded",
        "media_key_sha256": raw_ref.media_key.map(native_md_sha256_hex),
        "media_key_present": raw_ref.media_key.is_some(),
        "media_key_timestamp": raw_ref.media_key_timestamp,
        "database_policy": "metadata_hashes_only_no_media_key_direct_path_url_or_bytes",
        "event_policy": "secret_ref_and_hashes_only",
        "host_vault_payload": "raw_provider_download_ref_encrypted",
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_download_secret_payload(
    account_id: &str,
    fingerprint: &str,
    raw_ref: &NativeMdMediaDownloadRefRaw<'_>,
) -> Value {
    json!({
        "version": 1,
        "provider": "whatsapp",
        "provider_shape": "whatsapp_native_md",
        "runtime_driver": "wa-rs",
        "account_id": account_id,
        "secret_purpose": "whatsapp_media_download_ref",
        "provider_media_ref_fingerprint": fingerprint,
        "media_type": raw_ref.media_type,
        "content_type": raw_ref.content_type,
        "file_length": raw_ref.file_length,
        "file_sha256_base64": raw_ref.file_sha256.map(|value| BASE64_STANDARD.encode(value)),
        "file_enc_sha256_base64": raw_ref.file_enc_sha256.map(|value| BASE64_STANDARD.encode(value)),
        "direct_path": raw_ref.direct_path,
        "static_url": raw_ref.static_url,
        "media_key_base64": raw_ref.media_key.map(|value| BASE64_STANDARD.encode(value)),
        "media_key_timestamp": raw_ref.media_key_timestamp,
        "payload_policy": "host_vault_only_not_postgres_events_logs_frontend",
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_message_info(info: &wa_rs::types::message::MessageInfo) -> Value {
    json!({
        "source": native_md_wa_rs_sanitized_message_source(&info.source),
        "message_id": info.id,
        "server_id": info.server_id,
        "message_type": info.r#type,
        "timestamp": info.timestamp.to_rfc3339(),
        "category": info.category,
        "multicast": info.multicast,
        "media_type": info.media_type,
        "edit": format!("{:?}", info.edit),
        "bot_info_present": info.bot_info.is_some(),
        "verified_name_present": info.verified_name.is_some(),
        "device_sent_meta_present": info.device_sent_meta.is_some(),
        "push_name_present": !info.push_name.trim().is_empty(),
        "meta_info": {
            "target_id": info.meta_info.target_id,
            "target_sender_jid": info.meta_info.target_sender.as_ref().map(native_md_jid_string),
            "deprecated_lid_session": info.meta_info.deprecated_lid_session,
            "thread_message_id": info.meta_info.thread_message_id,
            "thread_message_sender_jid": info.meta_info.thread_message_sender_jid.as_ref().map(native_md_jid_string),
        },
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_sanitized_message_source(
    source: &wa_rs::types::message::MessageSource,
) -> Value {
    json!({
        "chat_jid": native_md_jid_string(&source.chat),
        "sender_jid": native_md_jid_string(&source.sender),
        "is_from_me": source.is_from_me,
        "is_group": source.is_group,
        "addressing_mode": source.addressing_mode.map(|mode| mode.as_str()),
        "sender_alt_jid": source.sender_alt.as_ref().map(native_md_jid_string),
        "recipient_alt_jid": source.recipient_alt.as_ref().map(native_md_jid_string),
        "broadcast_list_owner_jid": source.broadcast_list_owner.as_ref().map(native_md_jid_string),
        "recipient_jid": source.recipient.as_ref().map(native_md_jid_string),
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_jid_string(jid: &wa_rs::Jid) -> String {
    jid.to_string()
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn classify_wa_rs_message_event(
    message: &wa_rs::wa_rs_proto::whatsapp::Message,
    info: &wa_rs::types::message::MessageInfo,
) -> NativeMdProviderEventFamily {
    use wa_rs::types::message::EditAttribute;

    if message.reaction_message.is_some() || message.enc_reaction_message.is_some() {
        return NativeMdProviderEventFamily::Reaction;
    }
    if message.call.is_some() || message.scheduled_call_creation_message.is_some() {
        return NativeMdProviderEventFamily::CallMetadata;
    }
    if message_has_media_payload(message) {
        return NativeMdProviderEventFamily::Media;
    }
    if message.edited_message.is_some() {
        return NativeMdProviderEventFamily::MessageUpdate;
    }
    if let Some(protocol_message) = message.protocol_message.as_ref()
        && matches!(
            protocol_message.r#type,
            Some(value)
                if value
                    == wa_rs::wa_rs_proto::whatsapp::message::protocol_message::Type::Revoke as i32
                    || value
                        == wa_rs::wa_rs_proto::whatsapp::message::protocol_message::Type::MessageEdit as i32
        )
    {
        return if protocol_message.r#type
            == Some(wa_rs::wa_rs_proto::whatsapp::message::protocol_message::Type::Revoke as i32)
        {
            NativeMdProviderEventFamily::MessageDelete
        } else {
            NativeMdProviderEventFamily::MessageUpdate
        };
    }

    match &info.edit {
        EditAttribute::MessageEdit | EditAttribute::AdminEdit | EditAttribute::PinInChat => {
            NativeMdProviderEventFamily::MessageUpdate
        }
        EditAttribute::SenderRevoke | EditAttribute::AdminRevoke => {
            NativeMdProviderEventFamily::MessageDelete
        }
        EditAttribute::Empty | EditAttribute::Unknown(_) if !info.media_type.trim().is_empty() => {
            NativeMdProviderEventFamily::Media
        }
        EditAttribute::Empty | EditAttribute::Unknown(_) => NativeMdProviderEventFamily::Message,
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn message_has_media_payload(message: &wa_rs::wa_rs_proto::whatsapp::Message) -> bool {
    message.image_message.is_some()
        || message.document_message.is_some()
        || message.audio_message.is_some()
        || message.video_message.is_some()
        || message.sticker_message.is_some()
        || message.document_with_caption_message.is_some()
        || message.view_once_message.is_some()
        || message.view_once_message_v2.is_some()
        || message.view_once_message_v2_extension.is_some()
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn message_has_nested_media_payload(message: &wa_rs::wa_rs_proto::whatsapp::Message) -> bool {
    message.document_with_caption_message.is_some()
        || message.view_once_message.is_some()
        || message.view_once_message_v2.is_some()
        || message.view_once_message_v2_extension.is_some()
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_wa_rs_event_name(event: &wa_rs::types::events::Event) -> &'static str {
    use wa_rs::types::events::Event;

    match event {
        Event::Connected(_) => "Connected",
        Event::Disconnected(_) => "Disconnected",
        Event::PairSuccess(_) => "PairSuccess",
        Event::PairError(_) => "PairError",
        Event::LoggedOut(_) => "LoggedOut",
        Event::PairingQrCode { .. } => "PairingQrCode",
        Event::PairingCode { .. } => "PairingCode",
        Event::QrScannedWithoutMultidevice(_) => "QrScannedWithoutMultidevice",
        Event::ClientOutdated(_) => "ClientOutdated",
        Event::Message(_, _) => "Message",
        Event::Receipt(_) => "Receipt",
        Event::UndecryptableMessage(_) => "UndecryptableMessage",
        Event::Notification(_) => "Notification",
        Event::ChatPresence(_) => "ChatPresence",
        Event::Presence(_) => "Presence",
        Event::PictureUpdate(_) => "PictureUpdate",
        Event::UserAboutUpdate(_) => "UserAboutUpdate",
        Event::JoinedGroup(_) => "JoinedGroup",
        Event::GroupInfoUpdate { .. } => "GroupInfoUpdate",
        Event::ContactUpdate(_) => "ContactUpdate",
        Event::PushNameUpdate(_) => "PushNameUpdate",
        Event::SelfPushNameUpdated(_) => "SelfPushNameUpdated",
        Event::PinUpdate(_) => "PinUpdate",
        Event::MuteUpdate(_) => "MuteUpdate",
        Event::ArchiveUpdate(_) => "ArchiveUpdate",
        Event::MarkChatAsReadUpdate(_) => "MarkChatAsReadUpdate",
        Event::HistorySync(_) => "HistorySync",
        Event::OfflineSyncPreview(_) => "OfflineSyncPreview",
        Event::OfflineSyncCompleted(_) => "OfflineSyncCompleted",
        Event::DeviceListUpdate(_) => "DeviceListUpdate",
        Event::BusinessStatusUpdate(_) => "BusinessStatusUpdate",
        Event::StreamReplaced(_) => "StreamReplaced",
        Event::TemporaryBan(_) => "TemporaryBan",
        Event::ConnectFailure(_) => "ConnectFailure",
        Event::StreamError(_) => "StreamError",
    }
}

const NATIVE_MD_PROVIDER_EVENT_FAMILIES: &[NativeMdProviderEventFamily] = &[
    NativeMdProviderEventFamily::Authentication,
    NativeMdProviderEventFamily::RuntimeLifecycle,
    NativeMdProviderEventFamily::SyncLifecycle,
    NativeMdProviderEventFamily::Message,
    NativeMdProviderEventFamily::MessageUpdate,
    NativeMdProviderEventFamily::MessageDelete,
    NativeMdProviderEventFamily::Receipt,
    NativeMdProviderEventFamily::Reaction,
    NativeMdProviderEventFamily::Dialog,
    NativeMdProviderEventFamily::Participant,
    NativeMdProviderEventFamily::Presence,
    NativeMdProviderEventFamily::CallMetadata,
    NativeMdProviderEventFamily::Status,
    NativeMdProviderEventFamily::StatusView,
    NativeMdProviderEventFamily::StatusDelete,
    NativeMdProviderEventFamily::Media,
    NativeMdProviderEventFamily::MediaLifecycle,
    NativeMdProviderEventFamily::CommandReconciliation,
    NativeMdProviderEventFamily::Unsupported,
];

const NATIVE_MD_VERIFIED_PROVIDER_COMMANDS: &[&str] = &[
    "send_text",
    "reply",
    "forward",
    "edit",
    "delete",
    "react",
    "unreact",
    "mark_read",
    "leave_group",
    "send_media",
    "send_voice_note",
    "download_media",
];

const NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS: &[&str] = &[
    "publish_status",
    "archive",
    "unarchive",
    "mute",
    "unmute",
    "pin",
    "unpin",
    "join_group",
    "mark_unread",
];

const NATIVE_MD_PUBLIC_AVAILABILITY_GATE: &str = "blocked_until_manual_live_smoke";
const NATIVE_MD_COMMAND_EXECUTION_GATE: &str = "smoke_gated_provider_observed_reconciliation";
const NATIVE_MD_UNSUPPORTED_COMMAND_ERROR_CODE: &str = "native_md_command_kind_unsupported";

fn native_md_unsupported_command_error(
    command_kind: &str,
) -> Option<WhatsAppProviderCommandExecutionError> {
    let command_kind = command_kind.trim();
    NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS
        .contains(&command_kind)
        .then(|| {
            WhatsAppProviderCommandExecutionError::new(
                NATIVE_MD_UNSUPPORTED_COMMAND_ERROR_CODE,
                format!(
                    "native_md wa-rs execution preflight has no verified support for `{command_kind}`; failed_before_runtime_driver_lookup"
                ),
                None,
            )
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdRuntimeLiveCapabilities {
    qr_link: bool,
    pair_code_link: bool,
    session_restore: bool,
    inbound_events: bool,
    command_execution: bool,
    media_transfer: bool,
}

impl NativeMdRuntimeLiveCapabilities {
    const fn blocked() -> Self {
        Self {
            qr_link: false,
            pair_code_link: false,
            session_restore: false,
            inbound_events: false,
            command_execution: false,
            media_transfer: false,
        }
    }

    fn all_blocked(self) -> bool {
        !self.qr_link
            && !self.pair_code_link
            && !self.session_restore
            && !self.inbound_events
            && !self.command_execution
            && !self.media_transfer
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdSessionStorageBoundary {
    secret_purpose: &'static str,
    secret_store_kind: &'static str,
    database_policy: &'static str,
    sdk_sqlite_policy: &'static str,
    postgres_secret_policy: &'static str,
}

impl NativeMdSessionStorageBoundary {
    const fn account_scoped_host_vault() -> Self {
        Self {
            secret_purpose: "whatsapp_web_session_key",
            secret_store_kind: "host_vault",
            database_policy: "metadata_binding_only",
            sdk_sqlite_policy: "disabled",
            postgres_secret_policy: "forbidden",
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdWaRsStoreFamily {
    SignalStore,
    AppSyncStore,
    ProtocolStore,
    DeviceStore,
}

impl NativeMdWaRsStoreFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::SignalStore => "SignalStore",
            Self::AppSyncStore => "AppSyncStore",
            Self::ProtocolStore => "ProtocolStore",
            Self::DeviceStore => "DeviceStore",
        }
    }

    fn storage_class(self) -> &'static str {
        match self {
            Self::SignalStore => "cryptographic_session_material",
            Self::AppSyncStore => "app_state_sync_material",
            Self::ProtocolStore => "protocol_alignment_material",
            Self::DeviceStore => "device_identity_material",
        }
    }
}

const NATIVE_MD_WA_RS_STORE_FAMILIES: &[NativeMdWaRsStoreFamily] = &[
    NativeMdWaRsStoreFamily::SignalStore,
    NativeMdWaRsStoreFamily::AppSyncStore,
    NativeMdWaRsStoreFamily::ProtocolStore,
    NativeMdWaRsStoreFamily::DeviceStore,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdWaRsStoreManifest {
    backend_trait: &'static str,
    store_families: &'static [NativeMdWaRsStoreFamily],
    account_binding_secret_purpose: &'static str,
    secret_store_kind: &'static str,
    database_policy: &'static str,
    sdk_sqlite_policy: &'static str,
    postgres_secret_policy: &'static str,
}

impl NativeMdWaRsStoreManifest {
    const fn host_vault_backend(storage: NativeMdSessionStorageBoundary) -> Self {
        Self {
            backend_trait: "wa_rs::store::Backend",
            store_families: NATIVE_MD_WA_RS_STORE_FAMILIES,
            account_binding_secret_purpose: storage.secret_purpose,
            secret_store_kind: storage.secret_store_kind,
            database_policy: storage.database_policy,
            sdk_sqlite_policy: storage.sdk_sqlite_policy,
            postgres_secret_policy: storage.postgres_secret_policy,
        }
    }

    fn assert_host_vault_boundary(self) {
        debug_assert_eq!(self.backend_trait, "wa_rs::store::Backend");
        debug_assert_eq!(
            self.account_binding_secret_purpose,
            "whatsapp_web_session_key"
        );
        debug_assert_eq!(self.secret_store_kind, "host_vault");
        debug_assert_eq!(self.database_policy, "metadata_binding_only");
        debug_assert_eq!(self.sdk_sqlite_policy, "disabled");
        debug_assert_eq!(self.postgres_secret_policy, "forbidden");
        debug_assert!(
            self.store_families
                .contains(&NativeMdWaRsStoreFamily::SignalStore)
        );
        debug_assert!(
            self.store_families
                .contains(&NativeMdWaRsStoreFamily::AppSyncStore)
        );
        debug_assert!(
            self.store_families
                .contains(&NativeMdWaRsStoreFamily::ProtocolStore)
        );
        debug_assert!(
            self.store_families
                .contains(&NativeMdWaRsStoreFamily::DeviceStore)
        );
    }

    fn health_check(self) -> Value {
        let store_families = self
            .store_families
            .iter()
            .map(|family| {
                json!({
                    "family": family.as_str(),
                    "storage_class": family.storage_class(),
                    "payload_store": self.secret_store_kind,
                    "database_policy": self.database_policy,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "backend_trait": self.backend_trait,
            "required_store_families": store_families,
            "account_binding_secret_purpose": self.account_binding_secret_purpose,
            "secret_store_kind": self.secret_store_kind,
            "database_policy": self.database_policy,
            "sdk_sqlite_policy": self.sdk_sqlite_policy,
            "postgres_secret_policy": self.postgres_secret_policy,
            "restore_scope": "account_scoped",
        })
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone)]
struct NativeMdHostVaultBackend {
    account_id: String,
    secret_ref: String,
    vault: crate::vault::HostVault,
    state: Arc<std::sync::Mutex<NativeMdHostVaultBackendSnapshot>>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdHostVaultBackend {
    fn open(
        account_id: String,
        secret_ref: String,
        vault: crate::vault::HostVault,
    ) -> wa_rs::store::error::Result<Self> {
        let snapshot = Self::load_snapshot(&vault, &secret_ref)?;
        Ok(Self {
            account_id,
            secret_ref,
            vault,
            state: Arc::new(std::sync::Mutex::new(snapshot)),
        })
    }

    fn load_snapshot(
        vault: &crate::vault::HostVault,
        secret_ref: &str,
    ) -> wa_rs::store::error::Result<NativeMdHostVaultBackendSnapshot> {
        match vault.read_secret(secret_ref) {
            Ok(value) => serde_json::from_str(&value).or_else(|_| {
                Ok(NativeMdHostVaultBackendSnapshot::legacy_opaque_session_imported())
            }),
            Err(crate::vault::HostVaultError::MissingSecret { .. }) => {
                Ok(NativeMdHostVaultBackendSnapshot::default())
            }
            Err(error) => Err(native_md_store_error(error)),
        }
    }

    fn read_state<R>(
        &self,
        reader: impl FnOnce(&NativeMdHostVaultBackendSnapshot) -> R,
    ) -> wa_rs::store::error::Result<R> {
        let state = self.state.lock().map_err(|_| {
            wa_rs::store::error::StoreError::Database(
                "native_md host-vault backend state poisoned".to_owned(),
            )
        })?;
        Ok(reader(&state))
    }

    fn mutate_state<R>(
        &self,
        writer: impl FnOnce(&mut NativeMdHostVaultBackendSnapshot) -> R,
    ) -> wa_rs::store::error::Result<R> {
        let mut state = self.state.lock().map_err(|_| {
            wa_rs::store::error::StoreError::Database(
                "native_md host-vault backend state poisoned".to_owned(),
            )
        })?;
        let result = writer(&mut state);
        self.persist_snapshot(&state)?;
        Ok(result)
    }

    fn persist_snapshot(
        &self,
        snapshot: &NativeMdHostVaultBackendSnapshot,
    ) -> wa_rs::store::error::Result<()> {
        let value = serde_json::to_string(snapshot).map_err(native_md_serialization_error)?;
        let metadata = json!({
            "provider": "whatsapp",
            "provider_shape": "whatsapp_native_md",
            "runtime_driver": "wa-rs",
            "storage_boundary": "host_vault_snapshot",
            "payload_policy": "encrypted_session_material_only",
            "database_policy": "metadata_binding_only",
            "sdk_sqlite_policy": "disabled",
            "postgres_secret_policy": "forbidden",
            "schema_version": snapshot.schema_version,
        });
        self.vault
            .store_secret(
                &self.secret_ref,
                &value,
                crate::vault::SecretEntryContext {
                    entry_kind: "provider_account_session",
                    account_id: self.account_id.as_str(),
                    purpose: "whatsapp_web_session_key",
                    secret_kind: "session_material",
                    label: "WhatsApp native multi-device session",
                    metadata: &metadata,
                },
            )
            .map_err(native_md_store_error)
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct NativeMdHostVaultBackendSnapshot {
    schema_version: u16,
    backend_trait: String,
    storage_boundary: String,
    legacy_opaque_session_material_present: bool,
    identities: std::collections::BTreeMap<String, Vec<u8>>,
    sessions: std::collections::BTreeMap<String, Vec<u8>>,
    prekeys: std::collections::BTreeMap<u32, NativeMdHostVaultPreKey>,
    signed_prekeys: std::collections::BTreeMap<u32, Vec<u8>>,
    sender_keys: std::collections::BTreeMap<String, Vec<u8>>,
    sync_keys: std::collections::BTreeMap<String, wa_rs::store::AppStateSyncKey>,
    app_state_versions: std::collections::BTreeMap<String, wa_rs_core::appstate::hash::HashState>,
    mutation_macs: std::collections::BTreeMap<String, std::collections::BTreeMap<String, Vec<u8>>>,
    skdm_recipients: std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    lid_mappings: std::collections::BTreeMap<String, wa_rs::store::LidPnMappingEntry>,
    pn_to_lid: std::collections::BTreeMap<String, String>,
    base_keys: std::collections::BTreeMap<String, Vec<u8>>,
    device_lists: std::collections::BTreeMap<String, wa_rs::store::DeviceListRecord>,
    forget_sender_key_marks: std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    tc_tokens: std::collections::BTreeMap<String, wa_rs::store::TcTokenEntry>,
    device: Option<wa_rs_core::store::Device>,
    next_device_id: i32,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl Default for NativeMdHostVaultBackendSnapshot {
    fn default() -> Self {
        Self {
            schema_version: 1,
            backend_trait: "wa_rs::store::Backend".to_owned(),
            storage_boundary: "host_vault_encrypted_snapshot".to_owned(),
            legacy_opaque_session_material_present: false,
            identities: std::collections::BTreeMap::new(),
            sessions: std::collections::BTreeMap::new(),
            prekeys: std::collections::BTreeMap::new(),
            signed_prekeys: std::collections::BTreeMap::new(),
            sender_keys: std::collections::BTreeMap::new(),
            sync_keys: std::collections::BTreeMap::new(),
            app_state_versions: std::collections::BTreeMap::new(),
            mutation_macs: std::collections::BTreeMap::new(),
            skdm_recipients: std::collections::BTreeMap::new(),
            lid_mappings: std::collections::BTreeMap::new(),
            pn_to_lid: std::collections::BTreeMap::new(),
            base_keys: std::collections::BTreeMap::new(),
            device_lists: std::collections::BTreeMap::new(),
            forget_sender_key_marks: std::collections::BTreeMap::new(),
            tc_tokens: std::collections::BTreeMap::new(),
            device: None,
            next_device_id: 1,
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdHostVaultBackendSnapshot {
    fn legacy_opaque_session_imported() -> Self {
        Self {
            legacy_opaque_session_material_present: true,
            ..Self::default()
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct NativeMdHostVaultPreKey {
    record: Vec<u8>,
    uploaded: bool,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_bytes_key(bytes: &[u8]) -> String {
    native_md_hex(bytes)
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_hex(bytes: &[u8]) -> String {
    let mut key = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut key, "{byte:02x}");
    }
    key
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_base_key_id(address: &str, message_id: &str) -> String {
    format!("{address}::{message_id}")
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_serialization_error(error: serde_json::Error) -> wa_rs::store::error::StoreError {
    wa_rs::store::error::StoreError::Serialization(error.to_string())
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_store_error(error: impl std::fmt::Display) -> wa_rs::store::error::StoreError {
    wa_rs::store::error::StoreError::Database(error.to_string())
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_parse_jid(value: &str) -> wa_rs::store::error::Result<wa_rs::Jid> {
    value
        .parse::<wa_rs::Jid>()
        .map_err(|error| wa_rs::store::error::StoreError::Serialization(error.to_string()))
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[async_trait::async_trait]
impl wa_rs::store::SignalStore for NativeMdHostVaultBackend {
    async fn put_identity(&self, address: &str, key: [u8; 32]) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.identities.insert(address.to_owned(), key.to_vec());
        })
    }

    async fn load_identity(&self, address: &str) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        self.read_state(|state| state.identities.get(address).cloned())
    }

    async fn delete_identity(&self, address: &str) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.identities.remove(address);
        })
    }

    async fn get_session(&self, address: &str) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        self.read_state(|state| state.sessions.get(address).cloned())
    }

    async fn put_session(&self, address: &str, session: &[u8]) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.sessions.insert(address.to_owned(), session.to_vec());
        })
    }

    async fn delete_session(&self, address: &str) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.sessions.remove(address);
        })
    }

    async fn store_prekey(
        &self,
        id: u32,
        record: &[u8],
        uploaded: bool,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.prekeys.insert(
                id,
                NativeMdHostVaultPreKey {
                    record: record.to_vec(),
                    uploaded,
                },
            );
        })
    }

    async fn load_prekey(&self, id: u32) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        self.read_state(|state| state.prekeys.get(&id).map(|prekey| prekey.record.clone()))
    }

    async fn remove_prekey(&self, id: u32) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.prekeys.remove(&id);
        })
    }

    async fn store_signed_prekey(&self, id: u32, record: &[u8]) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.signed_prekeys.insert(id, record.to_vec());
        })
    }

    async fn load_signed_prekey(&self, id: u32) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        self.read_state(|state| state.signed_prekeys.get(&id).cloned())
    }

    async fn load_all_signed_prekeys(&self) -> wa_rs::store::error::Result<Vec<(u32, Vec<u8>)>> {
        self.read_state(|state| {
            state
                .signed_prekeys
                .iter()
                .map(|(id, record)| (*id, record.clone()))
                .collect()
        })
    }

    async fn remove_signed_prekey(&self, id: u32) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.signed_prekeys.remove(&id);
        })
    }

    async fn put_sender_key(
        &self,
        address: &str,
        record: &[u8],
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state
                .sender_keys
                .insert(address.to_owned(), record.to_vec());
        })
    }

    async fn get_sender_key(&self, address: &str) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        self.read_state(|state| state.sender_keys.get(address).cloned())
    }

    async fn delete_sender_key(&self, address: &str) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.sender_keys.remove(address);
        })
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[async_trait::async_trait]
impl wa_rs::store::AppSyncStore for NativeMdHostVaultBackend {
    async fn get_sync_key(
        &self,
        key_id: &[u8],
    ) -> wa_rs::store::error::Result<Option<wa_rs::store::AppStateSyncKey>> {
        let key = native_md_bytes_key(key_id);
        self.read_state(|state| state.sync_keys.get(&key).cloned())
    }

    async fn set_sync_key(
        &self,
        key_id: &[u8],
        key: wa_rs::store::AppStateSyncKey,
    ) -> wa_rs::store::error::Result<()> {
        let store_key = native_md_bytes_key(key_id);
        self.mutate_state(|state| {
            state.sync_keys.insert(store_key, key);
        })
    }

    async fn get_version(
        &self,
        name: &str,
    ) -> wa_rs::store::error::Result<wa_rs_core::appstate::hash::HashState> {
        self.read_state(|state| {
            state
                .app_state_versions
                .get(name)
                .cloned()
                .unwrap_or_default()
        })
    }

    async fn set_version(
        &self,
        name: &str,
        state_value: wa_rs_core::appstate::hash::HashState,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state
                .app_state_versions
                .insert(name.to_owned(), state_value);
        })
    }

    async fn put_mutation_macs(
        &self,
        name: &str,
        _version: u64,
        mutations: &[wa_rs_core::appstate::processor::AppStateMutationMAC],
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            let collection = state.mutation_macs.entry(name.to_owned()).or_default();
            for mutation in mutations {
                collection.insert(
                    native_md_bytes_key(&mutation.index_mac),
                    mutation.value_mac.clone(),
                );
            }
        })
    }

    async fn get_mutation_mac(
        &self,
        name: &str,
        index_mac: &[u8],
    ) -> wa_rs::store::error::Result<Option<Vec<u8>>> {
        let index_key = native_md_bytes_key(index_mac);
        self.read_state(|state| {
            state
                .mutation_macs
                .get(name)
                .and_then(|collection| collection.get(&index_key).cloned())
        })
    }

    async fn delete_mutation_macs(
        &self,
        name: &str,
        index_macs: &[Vec<u8>],
    ) -> wa_rs::store::error::Result<()> {
        let keys = index_macs
            .iter()
            .map(|index_mac| native_md_bytes_key(index_mac))
            .collect::<Vec<_>>();
        self.mutate_state(|state| {
            if let Some(collection) = state.mutation_macs.get_mut(name) {
                for key in keys {
                    collection.remove(&key);
                }
            }
        })
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[async_trait::async_trait]
impl wa_rs::store::ProtocolStore for NativeMdHostVaultBackend {
    async fn get_skdm_recipients(
        &self,
        group_jid: &str,
    ) -> wa_rs::store::error::Result<Vec<wa_rs::Jid>> {
        let recipient_strings = self.read_state(|state| {
            state
                .skdm_recipients
                .get(group_jid)
                .cloned()
                .unwrap_or_default()
        })?;
        recipient_strings
            .iter()
            .map(|jid| native_md_parse_jid(jid))
            .collect()
    }

    async fn add_skdm_recipients(
        &self,
        group_jid: &str,
        device_jids: &[wa_rs::Jid],
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            let recipients = state
                .skdm_recipients
                .entry(group_jid.to_owned())
                .or_default();
            for jid in device_jids {
                recipients.insert(jid.to_string());
            }
        })
    }

    async fn clear_skdm_recipients(&self, group_jid: &str) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.skdm_recipients.remove(group_jid);
        })
    }

    async fn get_lid_mapping(
        &self,
        lid: &str,
    ) -> wa_rs::store::error::Result<Option<wa_rs::store::LidPnMappingEntry>> {
        self.read_state(|state| state.lid_mappings.get(lid).cloned())
    }

    async fn get_pn_mapping(
        &self,
        phone: &str,
    ) -> wa_rs::store::error::Result<Option<wa_rs::store::LidPnMappingEntry>> {
        self.read_state(|state| {
            state
                .pn_to_lid
                .get(phone)
                .and_then(|lid| state.lid_mappings.get(lid))
                .cloned()
                .or_else(|| {
                    state
                        .lid_mappings
                        .values()
                        .filter(|entry| entry.phone_number == phone)
                        .max_by_key(|entry| entry.updated_at)
                        .cloned()
                })
        })
    }

    async fn put_lid_mapping(
        &self,
        entry: &wa_rs::store::LidPnMappingEntry,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state
                .pn_to_lid
                .insert(entry.phone_number.clone(), entry.lid.clone());
            state.lid_mappings.insert(entry.lid.clone(), entry.clone());
        })
    }

    async fn get_all_lid_mappings(
        &self,
    ) -> wa_rs::store::error::Result<Vec<wa_rs::store::LidPnMappingEntry>> {
        self.read_state(|state| state.lid_mappings.values().cloned().collect())
    }

    async fn save_base_key(
        &self,
        address: &str,
        message_id: &str,
        base_key: &[u8],
    ) -> wa_rs::store::error::Result<()> {
        let key = native_md_base_key_id(address, message_id);
        self.mutate_state(|state| {
            state.base_keys.insert(key, base_key.to_vec());
        })
    }

    async fn has_same_base_key(
        &self,
        address: &str,
        message_id: &str,
        current_base_key: &[u8],
    ) -> wa_rs::store::error::Result<bool> {
        let key = native_md_base_key_id(address, message_id);
        self.read_state(|state| {
            state
                .base_keys
                .get(&key)
                .is_some_and(|saved| saved.as_slice() == current_base_key)
        })
    }

    async fn delete_base_key(
        &self,
        address: &str,
        message_id: &str,
    ) -> wa_rs::store::error::Result<()> {
        let key = native_md_base_key_id(address, message_id);
        self.mutate_state(|state| {
            state.base_keys.remove(&key);
        })
    }

    async fn update_device_list(
        &self,
        record: wa_rs::store::DeviceListRecord,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.device_lists.insert(record.user.clone(), record);
        })
    }

    async fn get_devices(
        &self,
        user: &str,
    ) -> wa_rs::store::error::Result<Option<wa_rs::store::DeviceListRecord>> {
        self.read_state(|state| state.device_lists.get(user).cloned())
    }

    async fn mark_forget_sender_key(
        &self,
        group_jid: &str,
        participant: &str,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state
                .forget_sender_key_marks
                .entry(group_jid.to_owned())
                .or_default()
                .insert(participant.to_owned());
        })
    }

    async fn consume_forget_marks(
        &self,
        group_jid: &str,
    ) -> wa_rs::store::error::Result<Vec<String>> {
        self.mutate_state(|state| {
            state
                .forget_sender_key_marks
                .remove(group_jid)
                .unwrap_or_default()
                .into_iter()
                .collect()
        })
    }

    async fn get_tc_token(
        &self,
        jid: &str,
    ) -> wa_rs::store::error::Result<Option<wa_rs::store::TcTokenEntry>> {
        self.read_state(|state| state.tc_tokens.get(jid).cloned())
    }

    async fn put_tc_token(
        &self,
        jid: &str,
        entry: &wa_rs::store::TcTokenEntry,
    ) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.tc_tokens.insert(jid.to_owned(), entry.clone());
        })
    }

    async fn delete_tc_token(&self, jid: &str) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.tc_tokens.remove(jid);
        })
    }

    async fn get_all_tc_token_jids(&self) -> wa_rs::store::error::Result<Vec<String>> {
        self.read_state(|state| state.tc_tokens.keys().cloned().collect())
    }

    async fn delete_expired_tc_tokens(
        &self,
        cutoff_timestamp: i64,
    ) -> wa_rs::store::error::Result<u32> {
        self.mutate_state(|state| {
            let expired = state
                .tc_tokens
                .iter()
                .filter_map(|(jid, entry)| {
                    (entry.token_timestamp < cutoff_timestamp).then_some(jid.clone())
                })
                .collect::<Vec<_>>();
            let deleted = expired.len() as u32;
            for jid in expired {
                state.tc_tokens.remove(&jid);
            }
            deleted
        })
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[async_trait::async_trait]
impl wa_rs::store::DeviceStore for NativeMdHostVaultBackend {
    async fn save(&self, device: &wa_rs_core::store::Device) -> wa_rs::store::error::Result<()> {
        self.mutate_state(|state| {
            state.device = Some(device.clone());
        })
    }

    async fn load(&self) -> wa_rs::store::error::Result<Option<wa_rs_core::store::Device>> {
        self.read_state(|state| state.device.clone())
    }

    async fn exists(&self) -> wa_rs::store::error::Result<bool> {
        self.read_state(|state| state.device.is_some())
    }

    async fn create(&self) -> wa_rs::store::error::Result<i32> {
        self.mutate_state(|state| {
            let device_id = state.next_device_id;
            state.next_device_id += 1;
            state
                .device
                .get_or_insert_with(wa_rs_core::store::Device::new);
            device_id
        })
    }

    async fn snapshot_db(
        &self,
        _name: &str,
        _extra_content: Option<&[u8]>,
    ) -> wa_rs::store::error::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
struct NativeMdWaRsClientFactory;

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdWaRsClientFactory {
    #[allow(clippy::too_many_arguments)]
    fn configured_builder(
        account_id: String,
        secret_ref: String,
        secret_store: SecretReferenceStore,
        vault: crate::vault::HostVault,
        pair_phone_number: Option<String>,
        auth_artifacts: Arc<NativeMdTransientAuthArtifacts>,
        event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
        lifecycle: NativeMdRuntimeLifecycleRegistry,
    ) -> wa_rs::store::error::Result<wa_rs::bot::BotBuilder> {
        let backend = Arc::new(NativeMdHostVaultBackend::open(
            account_id.clone(),
            secret_ref,
            vault.clone(),
        )?) as Arc<dyn wa_rs::store::Backend>;
        let event_account_id = account_id.clone();
        let secret_store_for_handler = secret_store.clone();
        let vault_for_handler = vault.clone();
        let auth_artifacts_for_handler = auth_artifacts.clone();
        let event_sink_for_handler = event_sink.clone();
        let lifecycle_for_handler = lifecycle.clone();
        let mut builder = wa_rs::bot::Bot::builder()
            .with_backend(backend)
            .with_transport_factory(wa_rs::transport::TokioWebSocketTransportFactory::new())
            .with_http_client(wa_rs::transport::UreqHttpClient::new())
            .skip_history_sync()
            .on_event(move |event, _client| {
                let event_account_id = event_account_id.clone();
                let secret_store = secret_store_for_handler.clone();
                let vault = vault_for_handler.clone();
                let auth_artifacts = auth_artifacts_for_handler.clone();
                let event_sink = event_sink_for_handler.clone();
                let lifecycle = lifecycle_for_handler.clone();
                async move {
                    auth_artifacts.record_event(&event_account_id, &event).await;
                    let lifecycle_event = lifecycle
                        .record_provider_event(&event_account_id, &event)
                        .await;
                    let dto = native_md_owned_sanitized_runtime_event_dto(
                        &event_account_id,
                        &event,
                        &secret_store,
                        &vault,
                    )
                    .await;
                    if let Err(error) = event_sink.accept(dto).await {
                        tracing::warn!(
                            target: "hermes.whatsapp.native_md",
                            error_code = error.code,
                            "failed to enqueue sanitized wa-rs provider event"
                        );
                    }
                    if let Some(lifecycle_event) = lifecycle_event
                        && let Err(error) = event_sink
                            .accept(lifecycle_event.to_dto(&event_account_id))
                            .await
                    {
                        tracing::warn!(
                            target: "hermes.whatsapp.native_md",
                            error_code = error.code,
                            "failed to enqueue sanitized wa-rs lifecycle event"
                        );
                    }
                }
            });

        if let Some(phone_number) = pair_phone_number {
            builder = builder.with_pair_code(wa_rs::pair_code::PairCodeOptions {
                phone_number,
                show_push_notification: true,
                custom_code: None,
                platform_id: wa_rs::pair_code::PlatformId::OtherWebClient,
                platform_display: "Hermes Desktop".to_owned(),
            });
        }

        Ok(builder)
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
struct NativeMdLiveDriver {
    account_id: String,
    client: Arc<wa_rs::Client>,
    bot: wa_rs::bot::Bot,
    run_handle: Option<tokio::task::JoinHandle<()>>,
    event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdLiveDriverError {
    code: &'static str,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdLiveDriver {
    #[allow(clippy::too_many_arguments)]
    async fn build(
        account_id: String,
        secret_ref: String,
        secret_store: SecretReferenceStore,
        vault: crate::vault::HostVault,
        pair_phone_number: Option<String>,
        auth_artifacts: Arc<NativeMdTransientAuthArtifacts>,
        event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
        lifecycle: NativeMdRuntimeLifecycleRegistry,
    ) -> Result<Self, NativeMdLiveDriverError> {
        let builder = NativeMdWaRsClientFactory::configured_builder(
            account_id.clone(),
            secret_ref,
            secret_store,
            vault,
            pair_phone_number,
            auth_artifacts,
            event_sink.clone(),
            lifecycle,
        )
        .map_err(|_| NativeMdLiveDriverError {
            code: "native_md_builder_configuration_failed",
        })?;
        let bot = builder.build().await.map_err(|_| NativeMdLiveDriverError {
            code: "native_md_bot_build_failed",
        })?;
        let client = bot.client();
        Ok(Self {
            account_id,
            client,
            bot,
            run_handle: None,
            event_sink,
        })
    }

    async fn start(&mut self) -> Result<(), NativeMdLiveDriverError> {
        if self.run_handle.is_some() {
            return Ok(());
        }
        let handle = self.bot.run().await.map_err(|_| NativeMdLiveDriverError {
            code: "native_md_bot_run_failed",
        })?;
        self.run_handle = Some(handle);
        Ok(())
    }

    async fn stop(&mut self) {
        self.client.disconnect().await;
        if let Some(handle) = self.run_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl Drop for NativeMdLiveDriver {
    fn drop(&mut self) {
        if let Some(handle) = self.run_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
async fn native_md_execute_provider_command(
    client: Arc<wa_rs::Client>,
    command: &WhatsAppProviderExecutableCommand,
) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError> {
    if !client.is_connected() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_runtime_not_connected",
            "native_md runtime is not connected to the provider",
            Some(5),
        ));
    }
    if !client.is_logged_in() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_runtime_not_logged_in",
            "native_md runtime is not logged in with a restorable provider session",
            Some(15),
        ));
    }

    match command.command_kind.as_str() {
        "send_text" => {
            let chat = native_md_command_chat_jid(command)?;
            let text = native_md_payload_string(command, "text")?;
            let provider_request_id = client
                .send_message(chat, native_md_text_message(text))
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                json!({
                    "operation": "send_text",
                    "text": native_md_text_payload_metadata(command, "text"),
                }),
            ))
        }
        "reply" => {
            let chat = native_md_command_chat_jid(command)?;
            let text = native_md_payload_string(command, "text")?;
            let reply_to_provider_message_id = command
                .payload
                .get("reply_to_provider_message_id")
                .and_then(Value::as_str)
                .or(command.provider_message_id.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    native_md_missing_field(
                        command,
                        "reply_to_provider_message_id",
                        "native_md_reply_requires_provider_message_id",
                    )
                })?;
            let provider_request_id = client
                .send_message(
                    chat.clone(),
                    native_md_reply_message(text, &chat.to_string(), reply_to_provider_message_id),
                )
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                json!({
                    "operation": "reply",
                    "reply_to_provider_message_id": reply_to_provider_message_id,
                    "text": native_md_text_payload_metadata(command, "text"),
                }),
            ))
        }
        "forward" => {
            let chat = native_md_command_chat_jid(command)?;
            let text = native_md_payload_string(command, "text")?;
            let from_provider_chat_id = command
                .payload
                .get("from_provider_chat_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    native_md_missing_field(
                        command,
                        "from_provider_chat_id",
                        "native_md_forward_requires_source_provider_chat_id",
                    )
                })?;
            let from_provider_message_id = command
                .payload
                .get("from_provider_message_id")
                .and_then(Value::as_str)
                .or(command.provider_message_id.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    native_md_missing_field(
                        command,
                        "from_provider_message_id",
                        "native_md_forward_requires_source_provider_message_id",
                    )
                })?;
            let provider_request_id = client
                .send_message(chat, native_md_forward_text_message(text))
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                json!({
                    "operation": "forward",
                    "submission_mode": "forwarded_text_reemit",
                    "from_provider_chat_id": from_provider_chat_id,
                    "from_provider_message_id": from_provider_message_id,
                    "context_info": {
                        "is_forwarded": true,
                        "forwarding_score": 1,
                    },
                    "text": native_md_text_payload_metadata(command, "text"),
                }),
            ))
        }
        "edit" => {
            let chat = native_md_command_chat_jid(command)?;
            let provider_message_id = native_md_provider_message_id(
                command,
                "native_md_edit_requires_provider_message_id",
            )?;
            let text = native_md_payload_string(command, "text")?;
            let provider_request_id = client
                .edit_message(
                    chat,
                    provider_message_id.clone(),
                    native_md_text_message(text),
                )
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                json!({
                    "operation": "edit",
                    "target_provider_message_id": provider_message_id,
                    "text": native_md_text_payload_metadata(command, "text"),
                }),
            ))
        }
        "delete" => {
            let chat = native_md_command_chat_jid(command)?;
            let provider_message_id = native_md_provider_message_id(
                command,
                "native_md_delete_requires_provider_message_id",
            )?;
            client
                .revoke_message(chat, provider_message_id.clone(), wa_rs::RevokeType::Sender)
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                None,
                json!({
                    "operation": "delete",
                    "target_provider_message_id": provider_message_id,
                    "revoke_type": "sender",
                }),
            ))
        }
        "react" | "unreact" => {
            let chat = native_md_command_chat_jid(command)?;
            let provider_message_id = native_md_provider_message_id(
                command,
                "native_md_reaction_requires_provider_message_id",
            )?;
            let reaction = if command.command_kind == "react" {
                native_md_payload_string(command, "reaction_emoji")?
            } else {
                String::new()
            };
            let provider_request_id = client
                .send_message(
                    chat.clone(),
                    native_md_reaction_message(&chat.to_string(), &provider_message_id, reaction),
                )
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                json!({
                    "operation": command.command_kind,
                    "target_provider_message_id": provider_message_id,
                    "reaction_payload_policy": "emoji_value_sent_to_provider_not_stored_in_submission",
                }),
            ))
        }
        "mark_read" => {
            let chat = native_md_command_chat_jid(command)?;
            let message_ids = native_md_mark_read_message_ids(command)?;
            client
                .mark_as_read(&chat, None, message_ids.clone())
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                None,
                json!({
                    "operation": "mark_read",
                    "message_id_count": message_ids.len(),
                    "sender_scope": "unspecified",
                }),
            ))
        }
        "download_media" => {
            let download_ref = native_md_media_download_ref(command)?;
            let media_type = native_md_media_download_type(download_ref)?;
            let bytes = client
                .download_from_params(
                    &download_ref.direct_path,
                    &download_ref.media_key,
                    &download_ref.file_sha256,
                    &download_ref.file_enc_sha256,
                    download_ref.file_length,
                    media_type,
                )
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(30)))?;
            Ok(native_md_downloaded_media_outcome(
                command,
                download_ref,
                bytes,
            ))
        }
        "send_media" | "send_voice_note" => {
            let chat = native_md_command_chat_jid(command)?;
            let upload_kind = native_md_media_upload_kind(command)?;
            let media_bytes = native_md_media_upload_bytes(command)?;
            let upload = client
                .upload(media_bytes, upload_kind.media_type())
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(30)))?;
            let provider_request_id = client
                .send_message(
                    chat,
                    native_md_media_upload_message(command, upload_kind, &upload)?,
                )
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(15)))?;
            Ok(native_md_submitted_outcome(
                command,
                Some(provider_request_id),
                native_md_media_upload_operation_metadata(command, upload_kind, &upload),
            ))
        }
        "leave_group" => {
            let chat = native_md_command_chat_jid(command)?;
            client
                .groups()
                .leave(&chat)
                .await
                .map_err(|_| native_md_sdk_failure(command, Some(30)))?;
            Ok(native_md_submitted_outcome(
                command,
                None,
                json!({
                    "operation": "leave_group",
                    "group_jid": chat.to_string(),
                }),
            ))
        }
        unsupported => Err(native_md_unsupported_command_error(unsupported).unwrap_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_command_kind_unverified",
                format!(
                    "native_md wa-rs execution boundary has no verified support for `{unsupported}`"
                ),
                None,
            )
        })),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_command_chat_jid(
    command: &WhatsAppProviderExecutableCommand,
) -> Result<wa_rs::Jid, WhatsAppProviderCommandExecutionError> {
    wa_rs::Jid::from_str(command.provider_chat_id.trim()).map_err(|_| {
        native_md_missing_field(
            command,
            "provider_chat_id",
            "native_md_invalid_provider_chat_jid",
        )
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_payload_string(
    command: &WhatsAppProviderExecutableCommand,
    field: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            native_md_missing_field(
                command,
                field,
                "native_md_command_missing_required_payload_field",
            )
        })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_provider_message_id(
    command: &WhatsAppProviderExecutableCommand,
    error_code: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    command
        .provider_message_id
        .as_deref()
        .or_else(|| {
            command
                .target_ref
                .get("provider_message_id")
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| native_md_missing_field(command, "provider_message_id", error_code))
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_mark_read_message_ids(
    command: &WhatsAppProviderExecutableCommand,
) -> Result<Vec<String>, WhatsAppProviderCommandExecutionError> {
    let values = command
        .payload
        .get("message_ids")
        .or_else(|| command.target_ref.get("message_ids"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            command
                .provider_message_id
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect()
        });
    if values.is_empty() {
        return Err(native_md_missing_field(
            command,
            "message_ids",
            "native_md_mark_read_requires_provider_message_ids",
        ));
    }
    Ok(values)
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_text_message(text: String) -> wa_rs::wa_rs_proto::whatsapp::Message {
    wa_rs::wa_rs_proto::whatsapp::Message {
        conversation: Some(text),
        ..Default::default()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_reply_message(
    text: String,
    remote_jid: &str,
    stanza_id: &str,
) -> wa_rs::wa_rs_proto::whatsapp::Message {
    use wa_rs::wa_rs_proto::whatsapp as wa;

    wa::Message {
        extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
            text: Some(text),
            context_info: Some(Box::new(wa::ContextInfo {
                stanza_id: Some(stanza_id.to_owned()),
                remote_jid: Some(remote_jid.to_owned()),
                ..Default::default()
            })),
            ..Default::default()
        })),
        ..Default::default()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_forward_text_message(text: String) -> wa_rs::wa_rs_proto::whatsapp::Message {
    use wa_rs::wa_rs_proto::whatsapp as wa;

    wa::Message {
        extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
            text: Some(text),
            context_info: Some(Box::new(wa::ContextInfo {
                forwarding_score: Some(1),
                is_forwarded: Some(true),
                ..Default::default()
            })),
            ..Default::default()
        })),
        ..Default::default()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_reaction_message(
    remote_jid: &str,
    provider_message_id: &str,
    reaction: String,
) -> wa_rs::wa_rs_proto::whatsapp::Message {
    use wa_rs::wa_rs_proto::whatsapp as wa;

    wa::Message {
        reaction_message: Some(wa::message::ReactionMessage {
            key: Some(wa::MessageKey {
                remote_jid: Some(remote_jid.to_owned()),
                id: Some(provider_message_id.to_owned()),
                ..Default::default()
            }),
            text: Some(reaction),
            sender_timestamp_ms: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdMediaUploadKind {
    Image,
    Video,
    Audio,
    Document,
    VoiceNote,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdMediaUploadKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Document => "document",
            Self::VoiceNote => "voice_note",
        }
    }

    fn media_type(self) -> wa_rs_core::download::MediaType {
        match self {
            Self::Image => wa_rs_core::download::MediaType::Image,
            Self::Video => wa_rs_core::download::MediaType::Video,
            Self::Audio | Self::VoiceNote => wa_rs_core::download::MediaType::Audio,
            Self::Document => wa_rs_core::download::MediaType::Document,
        }
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_upload_kind(
    command: &WhatsAppProviderExecutableCommand,
) -> Result<NativeMdMediaUploadKind, WhatsAppProviderCommandExecutionError> {
    if command.command_kind == "send_voice_note" {
        return Ok(NativeMdMediaUploadKind::VoiceNote);
    }

    let media_type = native_md_payload_string(command, "media_type")?;
    match media_type.trim().to_ascii_lowercase().as_str() {
        "image" => Ok(NativeMdMediaUploadKind::Image),
        "video" => Ok(NativeMdMediaUploadKind::Video),
        "audio" => Ok(NativeMdMediaUploadKind::Audio),
        "document" | "file" => Ok(NativeMdMediaUploadKind::Document),
        unsupported => Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_type_unsupported",
            format!("native_md media upload has no verified support for `{unsupported}`"),
            None,
        )),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_upload_bytes(
    command: &WhatsAppProviderExecutableCommand,
) -> Result<Vec<u8>, WhatsAppProviderCommandExecutionError> {
    let Some(media_bytes) = command.media_bytes.as_ref() else {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_bytes_missing",
            "native_md media upload requires in-memory media bytes prepared by the command worker",
            None,
        ));
    };
    if media_bytes.is_empty() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_bytes_empty",
            "native_md media upload cannot send an empty media blob",
            None,
        ));
    }
    Ok(media_bytes.clone_bytes())
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_download_ref(
    command: &WhatsAppProviderExecutableCommand,
) -> Result<&WhatsAppProviderMediaDownloadRef, WhatsAppProviderCommandExecutionError> {
    command.media_download_ref.as_ref().ok_or_else(|| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_media_download_ref_missing",
            "native_md media download requires a host-vault media ref prepared by the command worker",
            None,
        )
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_download_type(
    download_ref: &WhatsAppProviderMediaDownloadRef,
) -> Result<wa_rs_core::download::MediaType, WhatsAppProviderCommandExecutionError> {
    match download_ref.media_type.trim().to_ascii_lowercase().as_str() {
        "image" => Ok(wa_rs_core::download::MediaType::Image),
        "video" => Ok(wa_rs_core::download::MediaType::Video),
        "audio" | "voice_note" => Ok(wa_rs_core::download::MediaType::Audio),
        "document" | "file" => Ok(wa_rs_core::download::MediaType::Document),
        "sticker" => Ok(wa_rs_core::download::MediaType::Sticker),
        unsupported => Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_download_type_unsupported",
            format!("native_md media download has no verified support for `{unsupported}`"),
            None,
        )),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_upload_message(
    command: &WhatsAppProviderExecutableCommand,
    upload_kind: NativeMdMediaUploadKind,
    upload: &wa_rs::upload::UploadResponse,
) -> Result<wa_rs::wa_rs_proto::whatsapp::Message, WhatsAppProviderCommandExecutionError> {
    use wa_rs::wa_rs_proto::whatsapp as wa;

    let content_type = native_md_payload_string(command, "content_type")?;
    let caption = native_md_payload_optional_string(command, "caption");
    let filename = native_md_payload_optional_string(command, "filename")
        .unwrap_or_else(|| "attachment".to_owned());
    let media_key_timestamp = Some(Utc::now().timestamp());

    let message = match upload_kind {
        NativeMdMediaUploadKind::Image => wa::Message {
            image_message: Some(Box::new(wa::message::ImageMessage {
                mimetype: Some(content_type),
                caption,
                url: Some(upload.url.clone()),
                direct_path: Some(upload.direct_path.clone()),
                media_key: Some(upload.media_key.clone()),
                file_enc_sha256: Some(upload.file_enc_sha256.clone()),
                file_sha256: Some(upload.file_sha256.clone()),
                file_length: Some(upload.file_length),
                media_key_timestamp,
                ..Default::default()
            })),
            ..Default::default()
        },
        NativeMdMediaUploadKind::Video => wa::Message {
            video_message: Some(Box::new(wa::message::VideoMessage {
                mimetype: Some(content_type),
                caption,
                url: Some(upload.url.clone()),
                direct_path: Some(upload.direct_path.clone()),
                media_key: Some(upload.media_key.clone()),
                file_enc_sha256: Some(upload.file_enc_sha256.clone()),
                file_sha256: Some(upload.file_sha256.clone()),
                file_length: Some(upload.file_length),
                media_key_timestamp,
                ..Default::default()
            })),
            ..Default::default()
        },
        NativeMdMediaUploadKind::Audio | NativeMdMediaUploadKind::VoiceNote => wa::Message {
            audio_message: Some(Box::new(wa::message::AudioMessage {
                mimetype: Some(content_type),
                url: Some(upload.url.clone()),
                direct_path: Some(upload.direct_path.clone()),
                media_key: Some(upload.media_key.clone()),
                file_enc_sha256: Some(upload.file_enc_sha256.clone()),
                file_sha256: Some(upload.file_sha256.clone()),
                file_length: Some(upload.file_length),
                ptt: Some(upload_kind == NativeMdMediaUploadKind::VoiceNote),
                media_key_timestamp,
                ..Default::default()
            })),
            ..Default::default()
        },
        NativeMdMediaUploadKind::Document => wa::Message {
            document_message: Some(Box::new(wa::message::DocumentMessage {
                mimetype: Some(content_type),
                title: Some(filename.clone()),
                file_name: Some(filename),
                caption,
                url: Some(upload.url.clone()),
                direct_path: Some(upload.direct_path.clone()),
                media_key: Some(upload.media_key.clone()),
                file_enc_sha256: Some(upload.file_enc_sha256.clone()),
                file_sha256: Some(upload.file_sha256.clone()),
                file_length: Some(upload.file_length),
                media_key_timestamp,
                ..Default::default()
            })),
            ..Default::default()
        },
    };
    Ok(message)
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_media_upload_operation_metadata(
    command: &WhatsAppProviderExecutableCommand,
    upload_kind: NativeMdMediaUploadKind,
    upload: &wa_rs::upload::UploadResponse,
) -> Value {
    json!({
        "operation": command.command_kind,
        "media_upload": {
            "media_type": upload_kind.as_str(),
            "content_type": command.payload.get("content_type").cloned(),
            "filename": command.payload.get("filename").cloned(),
            "attachment_id": command.payload.get("attachment_id").cloned(),
            "blob_id": command.payload.get("blob_id").cloned(),
            "input_sha256": command.payload.get("sha256").cloned(),
            "input_size_bytes": command.media_bytes.as_ref().map(|bytes| bytes.len()),
            "caption": native_md_text_payload_metadata(command, "caption"),
            "provider_upload": {
                "file_length": upload.file_length,
                "file_sha256": native_md_digest_hex(&upload.file_sha256),
                "file_enc_sha256": native_md_digest_hex(&upload.file_enc_sha256),
                "direct_path": "excluded",
                "direct_path_sha256": native_md_sha256_hex(upload.direct_path.as_bytes()),
                "media_key": "excluded",
                "media_key_sha256": native_md_sha256_hex(&upload.media_key),
                "url": "excluded",
            },
            "bytes_policy": "in_memory_only_not_serialized_not_persisted",
            "completion_rule": "provider_observed_event_reconciliation_required",
        }
    })
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_payload_optional_string(
    command: &WhatsAppProviderExecutableCommand,
    field: &'static str,
) -> Option<String> {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_digest_hex(bytes: &[u8]) -> String {
    format!("sha256:{}", native_md_hex(bytes))
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_submitted_outcome(
    command: &WhatsAppProviderExecutableCommand,
    provider_request_id: Option<String>,
    operation_metadata: Value,
) -> WhatsAppProviderCommandExecutionOutcome {
    let submitted_at = Utc::now();
    let completion_target =
        native_md_provider_observed_completion_target(command, provider_request_id.as_deref());
    WhatsAppProviderCommandExecutionOutcome {
        command_id: command.command_id.clone(),
        provider_request_id: provider_request_id.clone(),
        result_payload: json!({
            "provider_submission": {
                "submitted": true,
                "submitted_at": submitted_at,
                "submitted_via": "native_md_wa_rs",
                "provider_shape": "whatsapp_native_md",
                "runtime_driver": "wa-rs",
                "command_kind": command.command_kind,
                "provider_request_id": provider_request_id.clone(),
                "provider_observed_completion_target": completion_target.clone(),
                "completion_rule": "provider_observed_event_reconciliation_required",
                "payload_policy": "sanitized_metadata_only",
                "message_body": "excluded",
                "media_bytes": "excluded",
                "session_material": "excluded",
                "raw_provider_payload": "excluded",
                "direct_domain_calls": "forbidden",
                "operation": operation_metadata,
            }
        }),
        provider_state: json!({
            "native_md": {
                "submitted": true,
                "submitted_at": submitted_at,
                "runtime_driver": "wa-rs",
                "provider_request_id": provider_request_id,
                "provider_observed_completion_target": completion_target,
                "reconciliation_status": "awaiting_provider_observed_event",
                "completion_rule": "provider_observed_event_reconciliation_required",
            }
        }),
        downloaded_media_bytes: None,
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_provider_observed_completion_target(
    command: &WhatsAppProviderExecutableCommand,
    provider_request_id: Option<&str>,
) -> Value {
    match command.command_kind.as_str() {
        "send_media" | "send_voice_note" => json!({
            "accepted_event_kind": "signal.accepted.whatsapp.media",
            "raw_record_kind": "whatsapp_web_media",
            "provider_message_id": provider_request_id,
            "provider_chat_id": command.provider_chat_id,
            "match_policy": "provider_request_id_equals_observed_media_provider_message_id",
            "fallback_match_policy": "blob_id_equals_observed_media_storage_path",
            "completion_rule": "provider_observed_event_reconciliation_required",
            "payload_policy": "sanitized_metadata_only",
            "media_bytes": "excluded",
            "raw_provider_payload": "excluded",
        }),
        _ => json!({
            "accepted_event_kind": "provider_specific_reconciliation_event",
            "provider_message_id": provider_request_id,
            "completion_rule": "provider_observed_event_reconciliation_required",
            "payload_policy": "sanitized_metadata_only",
        }),
    }
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_downloaded_media_outcome(
    command: &WhatsAppProviderExecutableCommand,
    download_ref: &WhatsAppProviderMediaDownloadRef,
    bytes: Vec<u8>,
) -> WhatsAppProviderCommandExecutionOutcome {
    let size_bytes = bytes.len();
    let downloaded_sha256 = native_md_digest_hex(&bytes);
    let mut outcome = native_md_submitted_outcome(
        command,
        None,
        json!({
            "operation": "download_media",
            "provider_media_ref_fingerprint": download_ref.provider_media_ref_fingerprint.as_str(),
            "media_type": download_ref.media_type.as_str(),
            "content_type": download_ref.content_type.as_str(),
            "provider_declared_file_length": download_ref.file_length,
            "downloaded_size_bytes": size_bytes,
            "downloaded_sha256": downloaded_sha256,
            "download_ref_secret_ref": download_ref.secret_ref.as_str(),
            "download_ref_secret_purpose": "whatsapp_media_download_ref",
            "local_blob_persistence": "required_by_application_worker",
            "direct_path": "excluded",
            "static_url": "excluded",
            "media_key": "excluded",
            "media_bytes": "in_memory_only_skip_serializing",
        }),
    );
    outcome.downloaded_media_bytes = Some(WhatsAppProviderInMemoryMediaBytes::new(bytes));
    outcome
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_text_payload_metadata(
    command: &WhatsAppProviderExecutableCommand,
    field: &'static str,
) -> Value {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .map(|text| {
            json!({
                "length": text.chars().count(),
                "sha256": native_md_sha256_hex(text.as_bytes()),
                "value": "excluded",
            })
        })
        .unwrap_or_else(|| json!({"value": "missing"}))
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_missing_field(
    command: &WhatsAppProviderExecutableCommand,
    field: &'static str,
    error_code: &'static str,
) -> WhatsAppProviderCommandExecutionError {
    WhatsAppProviderCommandExecutionError::new(
        error_code,
        format!(
            "native_md command `{}` is missing required `{field}`",
            command.command_kind
        ),
        None,
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_sdk_failure(
    command: &WhatsAppProviderExecutableCommand,
    retry_after_seconds: Option<i64>,
) -> WhatsAppProviderCommandExecutionError {
    WhatsAppProviderCommandExecutionError::new(
        "native_md_provider_sdk_command_failed",
        format!(
            "native_md provider SDK command `{}` failed; result metadata was redacted",
            command.command_kind
        ),
        retry_after_seconds,
    )
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;

    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", native_md_hex(&hasher.finalize()))
}

const NATIVE_MD_LIVE_SMOKE_OPT_IN_CONFIG_KEY: &str = "native_md_live_smoke_enabled";
const NATIVE_MD_LIVE_SMOKE_OPT_IN_ALIAS_CONFIG_KEY: &str = "whatsapp_native_md_live_smoke_enabled";

#[derive(Clone)]
pub(super) struct NativeMdRuntimeManager {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
    lifecycle: NativeMdRuntimeLifecycleRegistry,
    #[cfg(feature = "whatsapp-native-md-runtime")]
    auth_artifacts: Arc<NativeMdTransientAuthArtifacts>,
    #[cfg(feature = "whatsapp-native-md-runtime")]
    drivers: Arc<tokio::sync::Mutex<HashMap<String, NativeMdLiveDriver>>>,
}

impl NativeMdRuntimeManager {
    pub(super) fn new(
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
    ) -> Self {
        Self {
            provider_account_store,
            provider_secret_binding_store,
            event_sink,
            lifecycle: NativeMdRuntimeLifecycleRegistry::new(),
            #[cfg(feature = "whatsapp-native-md-runtime")]
            auth_artifacts: Arc::new(NativeMdTransientAuthArtifacts::new()),
            #[cfg(feature = "whatsapp-native-md-runtime")]
            drivers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub(super) async fn start_runtime(
        &self,
        inner: &dyn WhatsAppProviderRuntime,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppRuntimeStartRequest,
    ) -> Result<WhatsAppRuntimeStatus, WhatsappWebError> {
        let account = self.lookup_account(&request.account_id).await?;
        let status = inner
            .runtime_status(secret_store, vault, &request.account_id)
            .await?;
        if !native_md_live_smoke_opted_in(&account.config) {
            return Ok(native_md_blocked_status(
                status,
                "whatsapp_native_md_live_smoke_opt_in_required",
            ));
        }
        let Some(session_secret_ref) = status.session_secret_ref.clone() else {
            return Ok(native_md_blocked_status(
                status,
                "whatsapp_session_restore_unavailable",
            ));
        };
        self.start_account(
            request.account_id.clone(),
            session_secret_ref,
            secret_store.clone(),
            vault.clone(),
            None,
        )
        .await
        .map(|outcome| native_md_smoke_started_status(status, outcome.already_running))
        .map_err(|error| WhatsappWebError::InvalidRequest(error.code.to_owned()))
    }

    pub(super) async fn start_qr_link(
        &self,
        inner: &dyn WhatsAppProviderRuntime,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppQrLinkStartRequest,
    ) -> Result<WhatsAppQrLinkSession, WhatsappWebError> {
        let account = self.lookup_account(&request.account_id).await?;
        let session = inner.start_qr_link(secret_store, vault, request).await?;
        if !native_md_live_smoke_opted_in(&account.config) {
            return Ok(native_md_blocked_qr_link_session(
                session,
                "whatsapp_native_md_live_smoke_opt_in_required",
            ));
        }
        let session_secret_ref = self
            .ensure_link_session_binding(secret_store, vault, &account)
            .await?;
        self.clear_auth_artifact(&request.account_id).await;
        let outcome = self
            .start_account(
                request.account_id.clone(),
                session_secret_ref,
                secret_store.clone(),
                vault.clone(),
                None,
            )
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.code.to_owned()))?;
        let blocker = if outcome.already_running {
            "whatsapp_native_md_link_already_running_event_spine_pending"
        } else {
            "whatsapp_native_md_qr_link_started_event_spine_pending"
        };
        let artifact = self
            .wait_for_auth_artifact(
                &request.account_id,
                NativeMdTransientAuthArtifactKind::QrCode,
            )
            .await;
        native_md_started_qr_link_session(session, blocker, artifact)
    }

    pub(super) async fn start_pair_code_link(
        &self,
        inner: &dyn WhatsAppProviderRuntime,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &WhatsAppPairCodeStartRequest,
    ) -> Result<WhatsAppPairCodeSession, WhatsappWebError> {
        let account = self.lookup_account(&request.account_id).await?;
        let session = inner
            .start_pair_code_link(secret_store, vault, request)
            .await?;
        if !native_md_live_smoke_opted_in(&account.config) {
            return Ok(native_md_blocked_pair_code_link_session(
                session,
                "whatsapp_native_md_live_smoke_opt_in_required",
            ));
        }
        let session_secret_ref = self
            .ensure_link_session_binding(secret_store, vault, &account)
            .await?;
        self.clear_auth_artifact(&request.account_id).await;
        let outcome = self
            .start_account(
                request.account_id.clone(),
                session_secret_ref,
                secret_store.clone(),
                vault.clone(),
                Some(request.phone_number.clone()),
            )
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.code.to_owned()))?;
        let blocker = if outcome.already_running {
            "whatsapp_native_md_link_already_running_event_spine_pending"
        } else {
            "whatsapp_native_md_pair_code_link_started_event_spine_pending"
        };
        let artifact = self
            .wait_for_auth_artifact(
                &request.account_id,
                NativeMdTransientAuthArtifactKind::PairCode,
            )
            .await;
        Ok(native_md_started_pair_code_link_session(
            session, blocker, artifact,
        ))
    }

    pub(super) async fn stop_account(&self, account_id: &str) -> bool {
        self.stop_native_driver(account_id).await
    }

    pub(super) async fn execute_live_provider_command(
        &self,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let account = self
            .lookup_account(&command.account_id)
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "native_md_account_lookup_failed",
                    error.to_string(),
                    Some(30),
                )
            })?;
        if let Some(error) = native_md_unsupported_command_error(&command.command_kind) {
            return Err(error);
        }
        if !native_md_live_smoke_opted_in(&account.config) {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "whatsapp_native_md_live_smoke_opt_in_required",
                "native_md live execution requires explicit account smoke opt-in",
                None,
            ));
        }
        self.execute_native_command(command).await
    }

    pub(super) async fn decorate_runtime_health(
        &self,
        health: &mut WhatsAppRuntimeHealth,
        account_id: &str,
    ) {
        let manager_health = self.manager_health(account_id).await;
        health.checks["native_md_manager"] = manager_health.clone();
        health.checks["runtime"]["native_manager"] = manager_health;
    }

    async fn lookup_account(&self, account_id: &str) -> Result<ProviderAccount, WhatsappWebError> {
        self.provider_account_store
            .get(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{account_id}` is not configured"
                ))
            })
    }

    async fn maybe_account(&self, account_id: &str) -> Option<ProviderAccount> {
        self.provider_account_store
            .get(account_id)
            .await
            .ok()
            .flatten()
    }

    async fn manager_health(&self, account_id: &str) -> Value {
        let smoke_opt_in = self
            .maybe_account(account_id)
            .await
            .as_ref()
            .map(|account| native_md_live_smoke_opted_in(&account.config))
            .unwrap_or(false);
        let running = self.is_account_running(account_id).await;
        let active_account_count = self.active_account_count().await;
        json!({
            "wired": true,
            "account_scoped": true,
            "account_id": account_id,
            "running": running,
            "active_account_count": active_account_count,
            "start_policy": "explicit_account_config_smoke_opt_in",
            "smoke_opt_in": smoke_opt_in,
            "smoke_opt_in_config_key": NATIVE_MD_LIVE_SMOKE_OPT_IN_CONFIG_KEY,
            "smoke_opt_in_alias_config_key": NATIVE_MD_LIVE_SMOKE_OPT_IN_ALIAS_CONFIG_KEY,
            "requires_session_secret_ref": true,
            "link_start_creates_host_vault_binding": true,
            "session_secret_purpose": "whatsapp_web_session_key",
            "transient_auth_artifact_channel": "memory_only_not_postgres_events_logs",
            "transient_auth_artifact_response_scope": "start_request_only",
            "transient_auth_artifact_wait_seconds": NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_WAIT_SECONDS,
            "public_availability_gate": NATIVE_MD_PUBLIC_AVAILABILITY_GATE,
            "sdk_feature_enabled": cfg!(feature = "whatsapp-native-md-runtime"),
            "event_sink_wired": Arc::strong_count(&self.event_sink) >= 1,
            "event_sink_contract": "owned_sanitized_dto_event_spine_sink",
            "reconnect_policy": self.lifecycle.health(account_id).await,
            "provider_command_surface": native_md_provider_command_surface_health(),
            "direct_domain_calls": "forbidden",
        })
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn ensure_link_session_binding(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ProviderAccount,
    ) -> Result<String, WhatsappWebError> {
        let secret_ref = super::whatsapp_session_secret_ref(&account.account_id);
        let purpose = ProviderAccountSecretPurpose::WhatsappWebSessionKey;
        let metadata = json!({
            "provider": account.provider_kind.as_str(),
            "provider_shape": "whatsapp_native_md",
            "account_id": account.account_id,
            "secret_purpose": purpose.as_str(),
            "runtime": "native_md",
            "authorization_state": "linking",
            "runtime_driver": "wa-rs",
            "storage_boundary": "host_vault_snapshot",
            "payload_policy": "encrypted_session_material_only",
            "database_policy": "metadata_binding_only",
            "sdk_sqlite_policy": "disabled",
            "postgres_secret_policy": "forbidden",
        });
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::Other,
                    SecretStoreKind::HostVault,
                    format!(
                        "WhatsApp native multi-device session for {}",
                        account.account_id
                    ),
                )
                .metadata(metadata.clone()),
            )
            .await?;
        match vault.read_secret(&secret_ref) {
            Ok(_) => {}
            Err(crate::vault::HostVaultError::MissingSecret { .. }) => {
                let snapshot = serde_json::to_string(&NativeMdHostVaultBackendSnapshot::default())
                    .map_err(|error| {
                        WhatsappWebError::InvalidRequest(format!(
                            "native_md_session_snapshot_serialization_failed: {error}"
                        ))
                    })?;
                vault.store_secret(
                    &secret_ref,
                    &snapshot,
                    crate::vault::SecretEntryContext {
                        entry_kind: "provider_account_session",
                        account_id: account.account_id.as_str(),
                        purpose: purpose.as_str(),
                        secret_kind: SecretKind::Other.as_str(),
                        label: "WhatsApp native multi-device session",
                        metadata: &metadata,
                    },
                )?;
            }
            Err(error) => return Err(WhatsappWebError::HostVault(error)),
        }
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account.account_id,
                purpose,
                &secret_ref,
            ))
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
        Ok(secret_ref)
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn ensure_link_session_binding(
        &self,
        _secret_store: &SecretReferenceStore,
        _vault: &HostVault,
        _account: &ProviderAccount,
    ) -> Result<String, WhatsappWebError> {
        Err(WhatsappWebError::InvalidRequest(
            "whatsapp_native_md_runtime_feature_disabled".to_owned(),
        ))
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn start_account(
        &self,
        account_id: String,
        session_secret_ref: String,
        secret_store: SecretReferenceStore,
        vault: HostVault,
        pair_phone_number: Option<String>,
    ) -> Result<NativeMdRuntimeManagerStartOutcome, NativeMdRuntimeManagerError> {
        let reconnect_due = self.lifecycle.reconnect_due(&account_id, Utc::now()).await;
        let stale_driver = {
            let mut drivers = self.drivers.lock().await;
            if drivers.contains_key(&account_id) && !reconnect_due {
                return Ok(NativeMdRuntimeManagerStartOutcome {
                    already_running: true,
                });
            }
            if reconnect_due {
                drivers.remove(&account_id)
            } else {
                None
            }
        };

        if let Some(mut driver) = stale_driver {
            self.emit_lifecycle_event(
                &account_id,
                self.lifecycle.record_reconnect_started(&account_id).await,
            )
            .await;
            driver.stop().await;
        } else {
            self.emit_lifecycle_event(
                &account_id,
                self.lifecycle.record_start_requested(&account_id).await,
            )
            .await;
        }

        let mut driver = match NativeMdLiveDriver::build(
            account_id.clone(),
            session_secret_ref,
            secret_store,
            vault,
            pair_phone_number,
            self.auth_artifacts.clone(),
            self.event_sink.clone(),
            self.lifecycle.clone(),
        )
        .await
        {
            Ok(driver) => driver,
            Err(error) => {
                let manager_error = NativeMdRuntimeManagerError::from_driver(error);
                self.emit_lifecycle_event(
                    &account_id,
                    if reconnect_due {
                        self.lifecycle
                            .record_reconnect_failed(&account_id, manager_error.code)
                            .await
                    } else {
                        self.lifecycle
                            .record_start_failed(&account_id, manager_error.code)
                            .await
                    },
                )
                .await;
                return Err(manager_error);
            }
        };
        if let Err(error) = driver.start().await {
            let manager_error = NativeMdRuntimeManagerError::from_driver(error);
            self.emit_lifecycle_event(
                &account_id,
                if reconnect_due {
                    self.lifecycle
                        .record_reconnect_failed(&account_id, manager_error.code)
                        .await
                } else {
                    self.lifecycle
                        .record_start_failed(&account_id, manager_error.code)
                        .await
                },
            )
            .await;
            return Err(manager_error);
        }

        let mut drivers = self.drivers.lock().await;
        if drivers.contains_key(&account_id) {
            driver.stop().await;
            return Ok(NativeMdRuntimeManagerStartOutcome {
                already_running: true,
            });
        }
        drivers.insert(account_id.clone(), driver);
        self.emit_lifecycle_event(
            &account_id,
            self.lifecycle.record_start_succeeded(&account_id).await,
        )
        .await;
        Ok(NativeMdRuntimeManagerStartOutcome {
            already_running: false,
        })
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn start_account(
        &self,
        _account_id: String,
        _session_secret_ref: String,
        _secret_store: SecretReferenceStore,
        _vault: HostVault,
        _pair_phone_number: Option<String>,
    ) -> Result<NativeMdRuntimeManagerStartOutcome, NativeMdRuntimeManagerError> {
        Err(NativeMdRuntimeManagerError {
            code: "whatsapp_native_md_runtime_feature_disabled",
        })
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn clear_auth_artifact(&self, account_id: &str) {
        self.auth_artifacts.clear(account_id).await;
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn clear_auth_artifact(&self, _account_id: &str) {}

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn wait_for_auth_artifact(
        &self,
        account_id: &str,
        kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        self.auth_artifacts.wait_for(account_id, kind).await
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn wait_for_auth_artifact(
        &self,
        _account_id: &str,
        _kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        None
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn execute_native_command(
        &self,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let client = {
            let drivers = self.drivers.lock().await;
            drivers
                .get(&command.account_id)
                .map(|driver| driver.client.clone())
        };
        let Some(client) = client else {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "native_md_runtime_not_running",
                "native_md runtime driver is not running for this account",
                Some(15),
            ));
        };
        native_md_execute_provider_command(client, command).await
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn execute_native_command(
        &self,
        _command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        Err(WhatsAppProviderCommandExecutionError::new(
            "whatsapp_native_md_runtime_feature_disabled",
            "native_md runtime feature is not enabled in this build",
            None,
        ))
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn stop_native_driver(&self, account_id: &str) -> bool {
        let driver = {
            let mut drivers = self.drivers.lock().await;
            drivers.remove(account_id)
        };
        if let Some(mut driver) = driver {
            driver.stop().await;
            self.emit_lifecycle_event(account_id, self.lifecycle.record_stopped(account_id).await)
                .await;
            return true;
        }
        false
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn stop_native_driver(&self, _account_id: &str) -> bool {
        false
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn is_account_running(&self, account_id: &str) -> bool {
        self.drivers.lock().await.contains_key(account_id)
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn is_account_running(&self, _account_id: &str) -> bool {
        false
    }

    #[cfg(feature = "whatsapp-native-md-runtime")]
    async fn active_account_count(&self) -> usize {
        self.drivers.lock().await.len()
    }

    #[cfg(not(feature = "whatsapp-native-md-runtime"))]
    async fn active_account_count(&self) -> usize {
        0
    }

    async fn emit_lifecycle_event(&self, account_id: &str, event: NativeMdRuntimeLifecycleEvent) {
        if let Err(error) = self.event_sink.accept(event.to_dto(account_id)).await {
            tracing::warn!(
                target: "hermes.whatsapp.native_md",
                error_code = error.code,
                account_id = account_id,
                "failed to enqueue native_md runtime lifecycle event"
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdRuntimeManagerStartOutcome {
    already_running: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdRuntimeManagerError {
    code: &'static str,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdRuntimeManagerError {
    fn from_driver(error: NativeMdLiveDriverError) -> Self {
        Self { code: error.code }
    }
}

fn native_md_live_smoke_opted_in(config: &Value) -> bool {
    native_md_config_bool(config, NATIVE_MD_LIVE_SMOKE_OPT_IN_CONFIG_KEY)
        || native_md_config_bool(config, NATIVE_MD_LIVE_SMOKE_OPT_IN_ALIAS_CONFIG_KEY)
}

fn native_md_config_bool(config: &Value, key: &str) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn native_md_blocked_status(
    mut status: WhatsAppRuntimeStatus,
    blocker: &'static str,
) -> WhatsAppRuntimeStatus {
    status.status = "blocked".to_owned();
    status.live_runtime_available = false;
    status.live_send_available = false;
    status.media_download_available = false;
    status.media_upload_available = false;
    status.qr_pairing_available = false;
    status.pair_code_available = false;
    if !status.runtime_blockers.iter().any(|item| item == blocker) {
        status.runtime_blockers.push(blocker.to_owned());
    }
    status.last_error = Some(blocker.to_owned());
    status.updated_at = Utc::now();
    status
}

fn native_md_blocked_qr_link_session(
    mut session: WhatsAppQrLinkSession,
    blocker: &'static str,
) -> WhatsAppQrLinkSession {
    session.status = "blocked".to_owned();
    session.qr_svg = None;
    session.expires_at = None;
    native_md_push_blocker(&mut session.runtime_blockers, blocker);
    session
}

fn native_md_started_qr_link_session(
    mut session: WhatsAppQrLinkSession,
    blocker: &'static str,
    artifact: Option<NativeMdTransientAuthArtifact>,
) -> Result<WhatsAppQrLinkSession, WhatsappWebError> {
    if let Some(artifact) = artifact {
        debug_assert_eq!(artifact.kind, NativeMdTransientAuthArtifactKind::QrCode);
        session.qr_svg = Some(native_md_render_transient_qr_svg(&artifact.value)?);
        session.expires_at = Some(artifact.expires_at);
        native_md_push_blocker(
            &mut session.runtime_blockers,
            "whatsapp_native_md_qr_link_artifact_transient",
        );
    } else {
        session.qr_svg = None;
        session.expires_at = None;
        native_md_push_blocker(
            &mut session.runtime_blockers,
            "whatsapp_native_md_qr_link_artifact_pending",
        );
    }
    native_md_push_blocker(&mut session.runtime_blockers, blocker);
    native_md_push_blocker(
        &mut session.runtime_blockers,
        "whatsapp_native_md_public_availability_blocked",
    );
    Ok(session)
}

fn native_md_blocked_pair_code_link_session(
    mut session: WhatsAppPairCodeSession,
    blocker: &'static str,
) -> WhatsAppPairCodeSession {
    session.status = "blocked".to_owned();
    session.pair_code = None;
    session.expires_at = None;
    native_md_push_blocker(&mut session.runtime_blockers, blocker);
    session
}

fn native_md_started_pair_code_link_session(
    mut session: WhatsAppPairCodeSession,
    blocker: &'static str,
    artifact: Option<NativeMdTransientAuthArtifact>,
) -> WhatsAppPairCodeSession {
    if let Some(artifact) = artifact {
        debug_assert_eq!(artifact.kind, NativeMdTransientAuthArtifactKind::PairCode);
        session.pair_code = Some(artifact.value);
        session.expires_at = Some(artifact.expires_at);
        native_md_push_blocker(
            &mut session.runtime_blockers,
            "whatsapp_native_md_pair_code_artifact_transient",
        );
    } else {
        session.pair_code = None;
        session.expires_at = None;
        native_md_push_blocker(
            &mut session.runtime_blockers,
            "whatsapp_native_md_pair_code_artifact_pending",
        );
    }
    native_md_push_blocker(&mut session.runtime_blockers, blocker);
    native_md_push_blocker(
        &mut session.runtime_blockers,
        "whatsapp_native_md_public_availability_blocked",
    );
    session
}

fn native_md_render_transient_qr_svg(payload: &str) -> Result<String, WhatsappWebError> {
    let code = qrcode::QrCode::new(payload.as_bytes()).map_err(|error| {
        WhatsappWebError::InvalidRequest(format!("native_md_qr_render_failed: {error}"))
    })?;
    Ok(code
        .render::<qrcode::render::svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}

fn native_md_push_blocker(blockers: &mut Vec<String>, blocker: &'static str) {
    if !blockers.iter().any(|item| item == blocker) {
        blockers.push(blocker.to_owned());
    }
}

fn native_md_smoke_started_status(
    mut status: WhatsAppRuntimeStatus,
    already_running: bool,
) -> WhatsAppRuntimeStatus {
    status.status = "degraded".to_owned();
    status.live_runtime_available = false;
    status.live_send_available = false;
    status.media_download_available = false;
    status.media_upload_available = false;
    status.qr_pairing_available = false;
    status.pair_code_available = false;
    let blocker = if already_running {
        "whatsapp_native_md_live_smoke_already_running_unverified"
    } else {
        "whatsapp_native_md_live_smoke_started_unverified"
    };
    native_md_push_blocker(&mut status.runtime_blockers, blocker);
    native_md_push_blocker(
        &mut status.runtime_blockers,
        "whatsapp_native_md_public_availability_blocked",
    );
    status.last_error = Some(blocker.to_owned());
    status.updated_at = Utc::now();
    status
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdSdkCompileProbe {
    bot_type: &'static str,
    builder_type: &'static str,
    event_type: &'static str,
    backend_trait_type: &'static str,
    host_vault_backend_type: &'static str,
    transport_factory_trait_type: &'static str,
    concrete_transport_factory_type: &'static str,
    http_client_trait_type: &'static str,
    concrete_http_client_type: &'static str,
    message_info_type: &'static str,
    pair_code_options_type: &'static str,
    device_type: &'static str,
    configured_builder_factory: &'static str,
    event_handler_policy: &'static str,
    event_sink_trait_type: &'static str,
    event_sink_contract: &'static str,
    live_driver_type: &'static str,
    live_driver_lifecycle_policy: &'static str,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
type NativeMdConfiguredBuilderFn = fn(
    String,
    String,
    SecretReferenceStore,
    crate::vault::HostVault,
    Option<String>,
    Arc<NativeMdTransientAuthArtifacts>,
    Arc<dyn WhatsAppRuntimeEventSink>,
    NativeMdRuntimeLifecycleRegistry,
) -> wa_rs::store::error::Result<wa_rs::bot::BotBuilder>;

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_sdk_compile_probe() -> NativeMdSdkCompileProbe {
    let builder = wa_rs::bot::Bot::builder().skip_history_sync();
    std::mem::drop(builder);
    let _event_classifier: fn(&wa_rs::types::events::Event) -> NativeMdProviderEventClassification =
        classify_wa_rs_event;
    let _raw_evidence_envelope_builder: for<'a> fn(
        &'a str,
        &'a str,
        &wa_rs::types::events::Event,
    ) -> NativeMdRawEvidenceEnvelope<'a> = native_md_wa_rs_raw_evidence_envelope;
    let _sanitized_dto_builder: for<'a> fn(
        &'a str,
        &'a str,
        &wa_rs::types::events::Event,
    ) -> NativeMdSanitizedProviderEventDto<'a> = native_md_wa_rs_sanitized_event_dto;
    let _host_vault_backend_open: fn(
        String,
        String,
        crate::vault::HostVault,
    ) -> wa_rs::store::error::Result<NativeMdHostVaultBackend> = NativeMdHostVaultBackend::open;
    let _configured_builder: NativeMdConfiguredBuilderFn =
        NativeMdWaRsClientFactory::configured_builder;
    let _live_driver_build = NativeMdLiveDriver::build;
    let _live_driver_start = NativeMdLiveDriver::start;
    let _live_driver_stop = NativeMdLiveDriver::stop;
    let _event_sink = NativeMdSanitizedEventCaptureSink::new();
    NativeMdSdkCompileProbe {
        bot_type: std::any::type_name::<wa_rs::bot::Bot>(),
        builder_type: std::any::type_name::<wa_rs::bot::BotBuilder>(),
        event_type: std::any::type_name::<wa_rs::types::events::Event>(),
        backend_trait_type: std::any::type_name::<dyn wa_rs::store::Backend>(),
        host_vault_backend_type: std::any::type_name::<NativeMdHostVaultBackend>(),
        transport_factory_trait_type: std::any::type_name::<dyn wa_rs::transport::TransportFactory>(
        ),
        concrete_transport_factory_type: std::any::type_name::<
            wa_rs::transport::TokioWebSocketTransportFactory,
        >(),
        http_client_trait_type: std::any::type_name::<dyn wa_rs::http::HttpClient>(),
        concrete_http_client_type: std::any::type_name::<wa_rs::transport::UreqHttpClient>(),
        message_info_type: std::any::type_name::<wa_rs::types::message::MessageInfo>(),
        pair_code_options_type: std::any::type_name::<wa_rs::pair_code::PairCodeOptions>(),
        device_type: std::any::type_name::<wa_rs::store::Device>(),
        configured_builder_factory: "NativeMdWaRsClientFactory::configured_builder",
        event_handler_policy: "sanitized_dto_only_no_domain_calls",
        event_sink_trait_type: std::any::type_name::<dyn WhatsAppRuntimeEventSink>(),
        event_sink_contract: "owned_sanitized_dto_event_spine_sink",
        live_driver_type: std::any::type_name::<NativeMdLiveDriver>(),
        live_driver_lifecycle_policy: "build_then_run_disconnect_abort_no_public_availability",
    }
}

#[cfg(not(feature = "whatsapp-native-md-runtime"))]
fn native_md_sdk_compile_probe() -> NativeMdSdkCompileProbe {
    NativeMdSdkCompileProbe {
        bot_type: "feature-disabled",
        builder_type: "feature-disabled",
        event_type: "feature-disabled",
        backend_trait_type: "feature-disabled",
        host_vault_backend_type: "feature-disabled",
        transport_factory_trait_type: "feature-disabled",
        concrete_transport_factory_type: "feature-disabled",
        http_client_trait_type: "feature-disabled",
        concrete_http_client_type: "feature-disabled",
        message_info_type: "feature-disabled",
        pair_code_options_type: "feature-disabled",
        device_type: "feature-disabled",
        configured_builder_factory: "feature-disabled",
        event_handler_policy: "feature-disabled",
        event_sink_trait_type: "feature-disabled",
        event_sink_contract: "feature-disabled",
        live_driver_type: "feature-disabled",
        live_driver_lifecycle_policy: "feature-disabled",
    }
}

fn assert_native_md_event_classification_contract() {
    for classification in [
        NativeMdProviderEventClassification::authentication("PairingCode"),
        NativeMdProviderEventClassification::runtime_lifecycle("Connected"),
        NativeMdProviderEventClassification::sync_lifecycle("HistorySync"),
        NativeMdProviderEventClassification::message(
            "Message",
            NativeMdProviderEventFamily::Message,
        ),
        NativeMdProviderEventClassification::message(
            "Message",
            NativeMdProviderEventFamily::MessageUpdate,
        ),
        NativeMdProviderEventClassification::message(
            "Message",
            NativeMdProviderEventFamily::MessageDelete,
        ),
        NativeMdProviderEventClassification::message(
            "Message",
            NativeMdProviderEventFamily::Reaction,
        ),
        NativeMdProviderEventClassification::message("Message", NativeMdProviderEventFamily::Media),
        NativeMdProviderEventClassification::message(
            "Message",
            NativeMdProviderEventFamily::CallMetadata,
        ),
        NativeMdProviderEventClassification::receipt("Receipt"),
        NativeMdProviderEventClassification::dialog("ArchiveUpdate"),
        NativeMdProviderEventClassification::participant("ContactUpdate"),
        NativeMdProviderEventClassification::presence("Presence"),
        NativeMdProviderEventClassification::unsupported("Notification"),
    ] {
        classification.assert_signal_hub_boundary();
        NativeMdRawEvidenceEnvelope::from_classification(
            "account_fixture",
            "provider_event_fixture",
            classification,
        )
        .assert_append_only_hub_contract();
        NativeMdSanitizedProviderEventDto::from_envelope(
            NativeMdRawEvidenceEnvelope::from_classification(
                "account_fixture",
                "provider_event_fixture",
                classification,
            ),
            json!({
                "payload_policy": "sanitized_metadata_only",
                "message_body": "excluded",
                "media_bytes": "excluded",
                "session_material": "excluded",
                "raw_provider_payload": "excluded",
            }),
        )
        .assert_sanitized_contract();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdRuntimeActorContract {
    actor_scope: &'static str,
    command_channel: NativeMdRuntimeCommandChannel,
    event_sink: NativeMdRuntimeEventSink,
    session_storage: NativeMdSessionStorageBoundary,
    store_manifest: NativeMdWaRsStoreManifest,
    event_families: &'static [NativeMdProviderEventFamily],
    live_capabilities: NativeMdRuntimeLiveCapabilities,
    sdk_compile_probe: NativeMdSdkCompileProbe,
}

impl NativeMdRuntimeActorContract {
    fn compile_only() -> Self {
        let session_storage = NativeMdSessionStorageBoundary::account_scoped_host_vault();
        Self {
            actor_scope: "account_scoped_runtime_actor",
            command_channel: NativeMdRuntimeCommandChannel::DurableOutbox,
            event_sink: NativeMdRuntimeEventSink::SignalHubRawEvidence,
            session_storage,
            store_manifest: NativeMdWaRsStoreManifest::host_vault_backend(session_storage),
            event_families: NATIVE_MD_PROVIDER_EVENT_FAMILIES,
            live_capabilities: NativeMdRuntimeLiveCapabilities::blocked(),
            sdk_compile_probe: native_md_sdk_compile_probe(),
        }
    }

    fn assert_event_first_boundary(self) {
        debug_assert_eq!(self.actor_scope, "account_scoped_runtime_actor");
        debug_assert_eq!(
            self.command_channel.as_str(),
            "durable_provider_command_outbox"
        );
        debug_assert_eq!(self.event_sink.as_str(), "signal_hub_raw_evidence");
        debug_assert_eq!(
            self.session_storage.secret_purpose,
            "whatsapp_web_session_key"
        );
        debug_assert_eq!(self.session_storage.secret_store_kind, "host_vault");
        debug_assert_eq!(
            self.session_storage.database_policy,
            "metadata_binding_only"
        );
        debug_assert_eq!(self.session_storage.sdk_sqlite_policy, "disabled");
        debug_assert_eq!(self.session_storage.postgres_secret_policy, "forbidden");
        self.store_manifest.assert_host_vault_boundary();
        debug_assert!(self.live_capabilities.all_blocked());
        debug_assert!(
            self.event_families
                .contains(&NativeMdProviderEventFamily::Unsupported)
        );
        debug_assert!(!self.sdk_compile_probe.bot_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.event_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.backend_trait_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.host_vault_backend_type.is_empty());
        debug_assert!(
            !self
                .sdk_compile_probe
                .transport_factory_trait_type
                .is_empty()
        );
        debug_assert!(
            !self
                .sdk_compile_probe
                .concrete_transport_factory_type
                .is_empty()
        );
        debug_assert!(!self.sdk_compile_probe.http_client_trait_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.concrete_http_client_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.device_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.configured_builder_factory.is_empty());
        debug_assert!(!self.sdk_compile_probe.event_handler_policy.is_empty());
        debug_assert!(!self.sdk_compile_probe.event_sink_trait_type.is_empty());
        debug_assert!(!self.sdk_compile_probe.event_sink_contract.is_empty());
        debug_assert!(!self.sdk_compile_probe.live_driver_type.is_empty());
        debug_assert!(
            !self
                .sdk_compile_probe
                .live_driver_lifecycle_policy
                .is_empty()
        );
        for family in self.event_families {
            debug_assert!(!family.as_str().is_empty());
        }
        assert_native_md_event_classification_contract();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdDriverReadiness {
    MissingCompileFeature,
    SmokeGatedUnverified,
}

impl NativeMdDriverReadiness {
    fn live_runtime_enabled(self) -> bool {
        false
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::MissingCompileFeature => "missing_compile_feature",
            Self::SmokeGatedUnverified => "smoke_gated_unverified_public_blocked",
        }
    }

    fn blocker(self) -> &'static str {
        match self {
            Self::MissingCompileFeature => "whatsapp_native_md_runtime_feature_disabled",
            Self::SmokeGatedUnverified => "whatsapp_native_md_public_availability_blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdRuntimeDriverDescriptor {
    driver_id: &'static str,
    readiness: NativeMdDriverReadiness,
    session_secret_purpose: &'static str,
    actor_contract: NativeMdRuntimeActorContract,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
fn native_md_driver_descriptor() -> NativeMdRuntimeDriverDescriptor {
    NativeMdRuntimeDriverDescriptor {
        driver_id: "wa-rs",
        readiness: NativeMdDriverReadiness::SmokeGatedUnverified,
        session_secret_purpose: "whatsapp_web_session_key",
        actor_contract: NativeMdRuntimeActorContract::compile_only(),
    }
}

#[cfg(not(feature = "whatsapp-native-md-runtime"))]
fn native_md_driver_descriptor() -> NativeMdRuntimeDriverDescriptor {
    NativeMdRuntimeDriverDescriptor {
        driver_id: "blocked",
        readiness: NativeMdDriverReadiness::MissingCompileFeature,
        session_secret_purpose: "whatsapp_web_session_key",
        actor_contract: NativeMdRuntimeActorContract::compile_only(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMdRuntimeActor {
    driver: NativeMdRuntimeDriverDescriptor,
}

impl NativeMdRuntimeActor {
    fn compile_only(driver: NativeMdRuntimeDriverDescriptor) -> Self {
        Self { driver }
    }

    fn driver_id(self) -> &'static str {
        self.driver.driver_id
    }

    fn readiness(self) -> &'static str {
        self.driver.readiness.as_str()
    }

    fn session_secret_purpose(self) -> &'static str {
        self.driver.session_secret_purpose
    }

    fn command_channel(self) -> &'static str {
        self.driver.actor_contract.command_channel.as_str()
    }

    fn event_sink(self) -> &'static str {
        self.driver.actor_contract.event_sink.as_str()
    }

    fn live_runtime_enabled(self) -> bool {
        self.driver.readiness.live_runtime_enabled()
    }

    fn runtime_feature_blocker(self) -> &'static str {
        self.driver.readiness.blocker()
    }

    fn assert_event_first_boundary(self) {
        self.driver.actor_contract.assert_event_first_boundary();
    }
}

pub(super) fn native_md_live_runtime_enabled() -> bool {
    NativeMdRuntimeActor::compile_only(native_md_driver_descriptor()).live_runtime_enabled()
}

pub(super) fn native_md_runtime_feature_blocker() -> &'static str {
    NativeMdRuntimeActor::compile_only(native_md_driver_descriptor()).runtime_feature_blocker()
}

pub(super) fn native_md_runtime_driver_health_check() -> Value {
    let actor = NativeMdRuntimeActor::compile_only(native_md_driver_descriptor());
    json!({
        "driver_id": actor.driver_id(),
        "readiness": actor.readiness(),
        "live_runtime_enabled": actor.live_runtime_enabled(),
        "runtime_blocker": actor.runtime_feature_blocker(),
        "actor_scope": actor.driver.actor_contract.actor_scope,
        "command_channel": actor.command_channel(),
        "event_sink": actor.event_sink(),
        "session_store": actor.driver.actor_contract.session_storage.secret_store_kind,
        "session_secret_purpose": actor.session_secret_purpose(),
        "database_policy": actor.driver.actor_contract.session_storage.database_policy,
        "sdk_sqlite_policy": actor.driver.actor_contract.session_storage.sdk_sqlite_policy,
        "postgres_secret_policy": actor.driver.actor_contract.session_storage.postgres_secret_policy,
        "host_vault_backend_type": actor.driver.actor_contract.sdk_compile_probe.host_vault_backend_type,
        "configured_builder_factory": actor.driver.actor_contract.sdk_compile_probe.configured_builder_factory,
        "transport_factory_type": actor.driver.actor_contract.sdk_compile_probe.concrete_transport_factory_type,
        "http_client_type": actor.driver.actor_contract.sdk_compile_probe.concrete_http_client_type,
        "event_handler_policy": actor.driver.actor_contract.sdk_compile_probe.event_handler_policy,
        "event_sink_contract": actor.driver.actor_contract.sdk_compile_probe.event_sink_contract,
        "live_driver_type": actor.driver.actor_contract.sdk_compile_probe.live_driver_type,
        "live_driver_lifecycle_policy": actor.driver.actor_contract.sdk_compile_probe.live_driver_lifecycle_policy,
        "wa_rs_store_manifest": actor.driver.actor_contract.store_manifest.health_check(),
        "provider_command_surface": native_md_provider_command_surface_health(),
        "raw_evidence_policy": "append_only_sanitized_metadata",
    })
}

fn native_md_provider_command_surface_health() -> Value {
    json!({
        "public_availability_gate": NATIVE_MD_PUBLIC_AVAILABILITY_GATE,
        "execution_gate": NATIVE_MD_COMMAND_EXECUTION_GATE,
        "command_channel": "durable_provider_command_outbox",
        "completion_rule": "provider_observed_event_reconciliation_required",
        "sdk_success_state": "submitted_awaiting_provider_observed_evidence",
        "verified_provider_command_subset": NATIVE_MD_VERIFIED_PROVIDER_COMMANDS,
        "unsupported_provider_commands": NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS,
        "wa_rs_sdk_command_gap": native_md_wa_rs_sdk_command_gap_health(),
        "media_upload_submission": "verified_sdk_path_smoke_gated",
        "media_download": "verified_sdk_download_path_smoke_gated",
        "status_publish": "unsupported_until_provider_status_api_smoke_verified",
        "dialog_state_writes": "unsupported_until_archive_mute_pin_unread_api_smoke_verified",
        "group_join": "unsupported_until_join_invite_api_smoke_verified",
        "forward": "verified_forwarded_text_reemit_submission_smoke_gated",
        "payload_policy": "sanitized_metadata_only",
        "message_body": "excluded",
        "media_bytes": "excluded",
        "session_material": "excluded",
        "direct_domain_calls": "forbidden",
    })
}

fn native_md_wa_rs_sdk_command_gap_health() -> Value {
    json!({
        "runtime_driver": "wa-rs",
        "driver_version": "0.2.0",
        "evidence_basis": "local_crate_source_public_api_inventory",
        "verified_sdk_methods": [
            {
                "command_kind": "send_text",
                "wa_rs_api": "Client::send_message",
                "source": "wa-rs-0.2.0/src/send.rs",
            },
            {
                "command_kind": "reply",
                "wa_rs_api": "Client::send_message",
                "source": "wa-rs-0.2.0/src/send.rs",
            },
            {
                "command_kind": "forward",
                "wa_rs_api": "Client::send_message + ExtendedTextMessage.ContextInfo",
                "source": "wa-rs-0.2.0/src/send.rs + wa-rs-proto-0.2.0/src/whatsapp.rs",
                "submission_mode": "forwarded_text_reemit",
                "required_payload": "text_from_communications_projection",
            },
            {
                "command_kind": "edit",
                "wa_rs_api": "Client::edit_message",
                "source": "wa-rs-0.2.0/src/client.rs",
            },
            {
                "command_kind": "delete",
                "wa_rs_api": "Client::revoke_message",
                "source": "wa-rs-0.2.0/src/send.rs",
            },
            {
                "command_kind": "react",
                "wa_rs_api": "Client::send_message",
                "source": "wa-rs-0.2.0/src/send.rs",
            },
            {
                "command_kind": "mark_read",
                "wa_rs_api": "Client::mark_as_read",
                "source": "wa-rs-0.2.0/src/receipt.rs",
            },
            {
                "command_kind": "leave_group",
                "wa_rs_api": "Client::groups().leave",
                "source": "wa-rs-0.2.0/src/features/groups.rs",
            },
            {
                "command_kind": "send_media",
                "wa_rs_api": "Client::upload + Client::send_message",
                "source": "wa-rs-0.2.0/src/upload.rs",
            },
            {
                "command_kind": "download_media",
                "wa_rs_api": "Client::download_from_params",
                "source": "wa-rs-0.2.0/src/download.rs",
            }
        ],
        "missing_safe_write_apis": [
            {
                "command_kind": "publish_status",
                "wa_rs_surface": "no public status_publish API found",
                "required_before_support": "provider_observed_status_smoke",
            },
            {
                "command_kind": "archive",
                "wa_rs_surface": "ArchiveUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "unarchive",
                "wa_rs_surface": "ArchiveUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "mute",
                "wa_rs_surface": "MuteUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "unmute",
                "wa_rs_surface": "MuteUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "pin",
                "wa_rs_surface": "PinUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "unpin",
                "wa_rs_surface": "PinUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "mark_unread",
                "wa_rs_surface": "MarkChatAsReadUpdate is inbound app-state dispatch only",
                "required_before_support": "verified_app_state_write_api_or_provider_smoke",
            },
            {
                "command_kind": "join_group",
                "wa_rs_surface": "groups API exposes create/add/remove/admin/link/leave but no join-by-invite API",
                "required_before_support": "verified_join_invite_api_or_provider_smoke",
            }
        ],
        "unsupported_execution_policy": {
            "worker_behavior": "claim_due_native_md_commands_for_execution_may_claim_to_write_structured_failure",
            "error_code": NATIVE_MD_UNSUPPORTED_COMMAND_ERROR_CODE,
            "event_phase": "failed_before_provider_observation",
            "retry_path": "terminal_dead_letter_without_retry",
            "completion_rule": "never_completed_without_provider_observed_event",
        },
        "public_availability": "blocked_until_manual_live_smoke_and_missing_safe_write_apis_are_resolved",
        "payload_policy": "sanitized_metadata_only",
        "direct_domain_calls": "forbidden",
    })
}

pub(crate) fn build_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    event_sink: Arc<dyn WhatsAppRuntimeEventSink>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    let native_md_actor = NativeMdRuntimeActor::compile_only(native_md_driver_descriptor());
    native_md_actor.assert_event_first_boundary();
    let _driver_id = native_md_actor.driver_id();
    let _session_secret_purpose = native_md_actor.session_secret_purpose();
    let native_md_manager = NativeMdRuntimeManager::new(
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        event_sink,
    );
    Arc::new(
        ShapedWhatsAppProviderRuntime::new(
            WhatsAppProviderRuntimeShape::NativeMultiDevice,
            Arc::new(WhatsappWebStore::new(
                pool,
                provider_account_store,
                provider_secret_binding_store,
                provider_channel_message_store,
            )),
        )
        .with_native_md_manager(native_md_manager),
    )
}
