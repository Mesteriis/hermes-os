use super::*;
use crate::application::provider_runtime_contracts::{
    WhatsAppProviderRuntimeShape, WhatsAppRuntimeStatus,
};

// ---------------------------------------------------------------------------
// WhatsApp capability model (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

impl WhatsAppCapabilityState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    SecretAccess,
}

impl WhatsAppActionClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::SecretAccess => "secret_access",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappProviderShapeStatus {
    pub(crate) provider_shape: String,
    pub(crate) status: String,
    pub(crate) reason: String,
}

impl WhatsappProviderShapeStatus {
    fn new(
        provider_shape: WhatsAppProviderRuntimeShape,
        status: WhatsAppCapabilityState,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            provider_shape: provider_shape.as_str().to_owned(),
            status: status.as_str().to_owned(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappCapabilityAccountScope {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: String,
    pub(crate) runtime_kind: String,
    pub(crate) lifecycle_state: String,
    pub(crate) live_runtime_available: bool,
    pub(crate) live_send_available: bool,
    pub(crate) media_download_available: bool,
    pub(crate) media_upload_available: bool,
}

impl WhatsappCapabilityAccountScope {
    fn from_runtime_status(status: &WhatsAppRuntimeStatus) -> Self {
        Self {
            account_id: status.account_id.clone(),
            provider_kind: status.provider_kind.clone(),
            provider_shape: status.provider_shape.clone(),
            runtime_kind: status.runtime_kind.clone(),
            lifecycle_state: status.status.clone(),
            live_runtime_available: status.live_runtime_available,
            live_send_available: status.live_send_available,
            media_download_available: status.media_download_available,
            media_upload_available: status.media_upload_available,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: String,
    pub(crate) provider_shapes: Vec<WhatsappProviderShapeStatus>,
    pub(crate) account_scope: Option<WhatsappCapabilityAccountScope>,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current(runtime_shape: WhatsAppProviderRuntimeShape) -> Self {
        Self::build(runtime_shape, None)
    }

    pub(crate) fn current_for_account(status: &WhatsAppRuntimeStatus) -> Self {
        let runtime_shape = match status.provider_shape.as_str() {
            "whatsapp_native_md" => WhatsAppProviderRuntimeShape::NativeMultiDevice,
            "whatsapp_business_cloud" => WhatsAppProviderRuntimeShape::BusinessCloud,
            _ => WhatsAppProviderRuntimeShape::WebCompanion,
        };
        Self::build(runtime_shape, Some(status))
    }

    fn build(
        runtime_shape: WhatsAppProviderRuntimeShape,
        status: Option<&WhatsAppRuntimeStatus>,
    ) -> Self {
        let account_scope = status.map(WhatsappCapabilityAccountScope::from_runtime_status);
        let runtime_mode = status
            .map(|item| item.runtime_kind.clone())
            .unwrap_or_else(|| "fixture".to_owned());
        let mut response = Self {
            version: "2.0",
            runtime_mode,
            provider_shapes: vec![
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::WebCompanion,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::WebCompanion,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::WebCompanion,
                    ),
                ),
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    ),
                ),
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::BusinessCloud,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::BusinessCloud,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::BusinessCloud,
                    ),
                ),
            ],
            account_scope,
            capabilities: whatsapp_capability_rows(),
            planned_features: vec![
                "live_runtime_execution",
                "native_md_runtime",
                "business_cloud_runtime",
                "live_media_transfer_progress",
                "live_presence_feed",
                "live_call_feed",
                "live_status_feed",
                "manual_smoke_test_checklist",
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "whatsapp_business_cloud_as_personal_provider",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let provider_shape = scope.provider_shape.as_str();

        for capability in &mut self.capabilities {
            match capability.capability.as_str() {
                "runtime.fixture" if runtime_kind != "fixture" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "This account does not use the fixture-only WhatsApp runtime.".to_owned();
                }
                "auth.qr_link_start" if provider_shape == "whatsapp_business_cloud" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use owner QR pairing.".to_owned();
                }
                "auth.pair_code_link_start" if provider_shape == "whatsapp_business_cloud" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use pair-code linking.".to_owned();
                }
                capability_name
                    if provider_shape == "whatsapp_business_cloud"
                        && is_whatsapp_business_cloud_personal_capability(capability_name) =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "Business Cloud accounts do not expose personal WhatsApp chat/runtime operations."
                        .to_owned();
                }
                capability_name
                    if provider_shape == "whatsapp_business_cloud"
                        && is_whatsapp_business_cloud_personal_observe_capability(
                            capability_name,
                        ) =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "Business Cloud accounts do not expose companion/native WhatsApp observation and projection surfaces."
                        .to_owned();
                }
                capability_name if is_whatsapp_business_platform_capability(capability_name) => {
                    if provider_shape == "whatsapp_business_cloud" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = "Business Cloud account shape is configured, but Hermes does not execute official Business Platform operations yet."
                            .to_owned();
                    } else {
                        capability.status =
                            WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                        capability.reason =
                            "This capability is only valid for whatsapp_business_cloud.".to_owned();
                    }
                }
                "sessions.manual_state" | "sessions.restore"
                    if provider_shape == "whatsapp_business_cloud" =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use companion session restore material."
                            .to_owned();
                }
                "sessions.restore" if matches!(lifecycle_state, "created" | "link_required") => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account has not completed WhatsApp session linking yet.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "revoked" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account was revoked and must be relinked before restore.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "removed" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account was removed from Hermes runtime control.".to_owned();
                }
                "auth.qr_link_start" if lifecycle_state == "pair_code_pending" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "Pair-code linking is already pending for this account.".to_owned();
                }
                "auth.pair_code_link_start" if lifecycle_state == "qr_pending" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "QR linking is already pending for this account.".to_owned();
                }
                "media.download"
                    if runtime_kind != "fixture" && !scope.media_download_available =>
                {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp media download."
                    );
                }
                capability_name
                    if matches!(capability_name, "media.upload_send" | "media.voice_send")
                        && runtime_kind != "fixture"
                        && !scope.media_upload_available =>
                {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp media upload."
                    );
                }
                capability_name if is_whatsapp_provider_write_capability(capability_name) => {
                    if lifecycle_state == "removed" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason =
                            "Removed accounts cannot execute WhatsApp provider commands."
                                .to_owned();
                    } else if matches!(
                        lifecycle_state,
                        "created"
                            | "link_required"
                            | "qr_pending"
                            | "pair_code_pending"
                            | "revoked"
                    ) {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = format!(
                            "WhatsApp provider commands are blocked while account state is `{lifecycle_state}`."
                        );
                    } else if runtime_kind != "fixture" && !scope.live_send_available {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = format!(
                            "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp provider execution."
                        );
                    }
                }
                capability_name if is_whatsapp_runtime_observe_capability(capability_name) => {
                    if lifecycle_state == "removed" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason =
                            "Removed accounts do not project runtime evidence anymore.".to_owned();
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilityStatus {
    pub(crate) capability: String,
    pub(crate) category: String,
    pub(crate) status: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_required: bool,
    pub(crate) closure_gate: bool,
    pub(crate) reason: String,
}

impl WhatsappCapabilityStatus {
    fn new(
        capability: &str,
        category: &str,
        status: WhatsAppCapabilityState,
        action_class: WhatsAppActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            capability: capability.to_owned(),
            category: category.to_owned(),
            status: status.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            confirmation_required,
            closure_gate,
            reason: reason.to_owned(),
        }
    }
}

fn whatsapp_capability_rows() -> Vec<WhatsappCapabilityStatus> {
    vec![
        WhatsappCapabilityStatus::new(
            "runtime.fixture",
            "runtime",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Fixture WhatsApp runtime, append-only evidence ingest and projection are available for CI and local validation.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sessions.manual_state",
            "sessions",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Companion session metadata is stored without raw session secrets in PostgreSQL.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sessions.restore",
            "sessions",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::SecretAccess,
            "Authorized session material can be restored from host vault bindings, but only the fixture/runtime-safe restore path is implemented today.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "auth.qr_link_start",
            "auth",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "QR link lifecycle state and sanitized events exist, but live QR material is not emitted yet.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "auth.pair_code_link_start",
            "auth",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Pair-code lifecycle state and sanitized events exist, but live pair-code material is not emitted yet.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sync.chats",
            "sync",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Projected WhatsApp conversations can be synced through the fixture/runtime-safe control surface.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sync.history",
            "sync",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Projected WhatsApp message history can be replayed through the shared Communications read model.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.read_projection",
            "messages",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Canonical Communications reads already serve WhatsApp messages, reply refs and forward refs.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "search.messages",
            "search",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Provider-neutral message search already returns WhatsApp projection data.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "search.media",
            "search",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Provider-neutral media search already returns projected WhatsApp attachments.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.send_text",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Durable outbox, audit metadata and provider-observed fixture reconciliation exist, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.reply",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reply commands use the durable provider outbox and canonical reply refs, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.forward",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Forward commands use the durable provider outbox and canonical forward refs, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.edit",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Observed message-version projection and fixture reconciliation exist, but live edit execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.delete",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::Destructive,
            "Observed tombstones and fixture reconciliation exist, but live delete execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.react",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reaction outbox rows and provider-observed fixture reconciliation exist, but live reaction execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.unreact",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reaction removal uses the same durable command/reconciliation path, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "media.upload_send",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Media upload/send commands preserve blob metadata and fixture reconciliation, but live transfer execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "media.download",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::Read,
            "Media download commands preserve blob/hash contracts and fixture reconciliation, but live transfer execution remains blocked.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "media.voice_send",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Voice-note sending shares the durable media outbox path, but live upload/execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.join_group",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Group join commands are durable and fixture-reconciled, but live provider execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.leave_group",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Group leave commands are durable and fixture-reconciled, but live provider execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.archive",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unarchive",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mute",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unmute",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.pin",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unpin",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mark_read",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Read-state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mark_unread",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Unread-state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "status.observe",
            "status",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Fixture status evidence already projects into canonical Communications and Timeline-facing events.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "status.publish",
            "status",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Status publish uses the durable provider outbox and observed fixture reconciliation, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "presence.observe",
            "presence",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Presence evidence projects through Signal Hub and canonical identity metadata for fixture/runtime-safe flows.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "calls.observe",
            "calls",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Call metadata evidence already flows into the event spine and Timeline replay contract.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.phone_numbers",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::Read,
            "Business Cloud phone-number asset semantics are reserved for the future official provider.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.waba_assets",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::Read,
            "Business Cloud WABA asset semantics are reserved for the future official provider.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.templates",
            "business",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Business Cloud template submission uses the official messages endpoint through the durable outbox; public availability still waits for smoke and webhook reconciliation.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.messages.send_text",
            "business",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Business Cloud text send submission uses the official messages endpoint through the durable outbox, but public availability waits for smoke and webhook reconciliation.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.webhooks",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::Read,
            "Business Cloud webhook semantics are reserved for the future official provider.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.media_endpoints",
            "business",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Business Cloud media upload/send uses the official media endpoint followed by the messages endpoint through the durable outbox; public availability still waits for smoke and webhook reconciliation.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.catalogs",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::Read,
            "Business Cloud product catalog semantics are reserved for the future official provider.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.flows",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::ProviderWrite,
            "Business Cloud flow semantics are reserved for the future official provider.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "business.policy_limits",
            "business",
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::Read,
            "Business Cloud rate-limit and policy semantics are reserved for the future official provider.",
            false,
            true,
        ),
    ]
}

fn is_whatsapp_provider_write_capability(capability: &str) -> bool {
    matches!(
        capability,
        "messages.send_text"
            | "messages.reply"
            | "messages.forward"
            | "messages.edit"
            | "messages.delete"
            | "messages.react"
            | "messages.unreact"
            | "media.upload_send"
            | "media.voice_send"
            | "conversations.join_group"
            | "conversations.leave_group"
            | "conversations.archive"
            | "conversations.unarchive"
            | "conversations.mute"
            | "conversations.unmute"
            | "conversations.pin"
            | "conversations.unpin"
            | "conversations.mark_read"
            | "conversations.mark_unread"
            | "status.publish"
            | "business.messages.send_text"
    )
}

fn is_whatsapp_runtime_observe_capability(capability: &str) -> bool {
    matches!(
        capability,
        "sync.chats"
            | "sync.history"
            | "messages.read_projection"
            | "search.messages"
            | "search.media"
            | "status.observe"
            | "presence.observe"
            | "calls.observe"
            | "media.download"
    )
}

fn is_whatsapp_business_cloud_personal_capability(capability: &str) -> bool {
    matches!(
        capability,
        "sync.chats"
            | "sync.history"
            | "messages.send_text"
            | "messages.reply"
            | "messages.forward"
            | "messages.edit"
            | "messages.delete"
            | "messages.react"
            | "messages.unreact"
            | "media.upload_send"
            | "media.download"
            | "media.voice_send"
            | "conversations.join_group"
            | "conversations.leave_group"
            | "conversations.archive"
            | "conversations.unarchive"
            | "conversations.mute"
            | "conversations.unmute"
            | "conversations.pin"
            | "conversations.unpin"
            | "conversations.mark_read"
            | "conversations.mark_unread"
            | "status.publish"
    )
}

fn is_whatsapp_business_cloud_personal_observe_capability(capability: &str) -> bool {
    matches!(
        capability,
        "messages.read_projection"
            | "search.messages"
            | "search.media"
            | "status.observe"
            | "presence.observe"
            | "calls.observe"
    )
}

fn is_whatsapp_business_platform_capability(capability: &str) -> bool {
    matches!(
        capability,
        "business.phone_numbers"
            | "business.waba_assets"
            | "business.templates"
            | "business.messages.send_text"
            | "business.webhooks"
            | "business.media_endpoints"
            | "business.catalogs"
            | "business.flows"
            | "business.policy_limits"
    )
}

fn provider_shape_summary_status(
    current_shape: WhatsAppProviderRuntimeShape,
    shape: WhatsAppProviderRuntimeShape,
) -> WhatsAppCapabilityState {
    match (current_shape, shape) {
        (
            WhatsAppProviderRuntimeShape::WebCompanion,
            WhatsAppProviderRuntimeShape::WebCompanion,
        ) => WhatsAppCapabilityState::Available,
        (
            WhatsAppProviderRuntimeShape::NativeMultiDevice,
            WhatsAppProviderRuntimeShape::NativeMultiDevice,
        )
        | (
            WhatsAppProviderRuntimeShape::BusinessCloud,
            WhatsAppProviderRuntimeShape::BusinessCloud,
        ) => WhatsAppCapabilityState::Blocked,
        _ => WhatsAppCapabilityState::Planned,
    }
}

fn provider_shape_summary_reason(
    current_shape: WhatsAppProviderRuntimeShape,
    shape: WhatsAppProviderRuntimeShape,
) -> &'static str {
    match (current_shape, shape) {
        (
            WhatsAppProviderRuntimeShape::WebCompanion,
            WhatsAppProviderRuntimeShape::WebCompanion,
        ) => "Visible desktop companion/Web runtime is the current WhatsApp foundation.",
        (
            WhatsAppProviderRuntimeShape::NativeMultiDevice,
            WhatsAppProviderRuntimeShape::NativeMultiDevice,
        ) => {
            "Native multi-device shape is configured for this account, but the live runtime is not enabled yet."
        }
        (
            WhatsAppProviderRuntimeShape::BusinessCloud,
            WhatsAppProviderRuntimeShape::BusinessCloud,
        ) => {
            "Business Cloud shape is configured for this account, but Hermes only exposes business-safe contract stubs today."
        }
        (_, WhatsAppProviderRuntimeShape::NativeMultiDevice) => {
            "Native multi-device runtime is isolated behind the replaceable runtime boundary but is not enabled yet."
        }
        (_, WhatsAppProviderRuntimeShape::BusinessCloud) => {
            "Business Cloud API remains a separate future provider shape and does not replace personal WhatsApp support."
        }
        _ => "Visible desktop companion/Web runtime is the current WhatsApp foundation.",
    }
}

#[derive(Deserialize)]
pub(crate) struct TelegramListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatListResponse {
    pub(crate) items: Vec<TelegramChat>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMessageListResponse {
    pub(crate) items: Vec<ProviderCommunicationMessage>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappWebListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebSessionListResponse {
    pub(crate) items: Vec<WhatsappWebSession>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebMessageListResponse {
    pub(crate) items: Vec<WhatsappWebMessage>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramReactionDeleteQuery {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) reaction_emoji: String,
    pub(crate) sender_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) command_id: Option<String>,
}
