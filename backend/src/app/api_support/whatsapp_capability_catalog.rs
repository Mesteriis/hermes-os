use crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderRuntimeShape;

use super::whatsapp_capabilities::{
    WhatsAppActionClass, WhatsAppCapabilityState, WhatsappCapabilityStatus,
};

pub(crate) fn whatsapp_capability_rows() -> Vec<WhatsappCapabilityStatus> {
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
            false,
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
            WhatsAppCapabilityState::Planned,
            WhatsAppActionClass::ProviderWrite,
            "Business Cloud template submission remains reserved until smoke and webhook reconciliation evidence close the public availability gate.",
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

pub(crate) fn is_whatsapp_provider_write_capability(capability: &str) -> bool {
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

pub(crate) fn is_whatsapp_runtime_observe_capability(capability: &str) -> bool {
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

pub(crate) fn is_whatsapp_business_cloud_personal_capability(capability: &str) -> bool {
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

pub(crate) fn is_whatsapp_business_cloud_personal_observe_capability(capability: &str) -> bool {
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

pub(crate) fn is_whatsapp_business_platform_capability(capability: &str) -> bool {
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

pub(crate) fn provider_shape_summary_status(
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

pub(crate) fn provider_shape_summary_reason(
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
