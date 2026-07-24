use serde::{Deserialize, Serialize};

pub const WHATSAPP_PLANNED_FEATURES: &[&str] = &[
    "live_runtime_execution",
    "live_media_transfer_progress",
    "live_presence_feed",
    "live_call_feed",
    "live_status_feed",
    "manual_smoke_test_checklist",
];

pub const WHATSAPP_UNSUPPORTED_FEATURES: &[&str] = &[
    "hidden_web_scraping",
    "bulk_messaging",
    "auto_messaging",
    "auto_dialing",
    "whatsapp_data_fine_tuning",
    "external_headless_browser_automation",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppActionClass {
    Read,
    ProviderWrite,
    Destructive,
    SecretAccess,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppCapability {
    pub capability: String,
    pub category: String,
    pub status: WhatsAppCapabilityState,
    pub action_class: WhatsAppActionClass,
    pub confirmation_required: bool,
    pub closure_gate: bool,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppCapabilityScope {
    pub runtime_kind: Option<String>,
    pub lifecycle_state: Option<String>,
    pub live_runtime_available: bool,
    pub live_send_available: bool,
    pub media_download_available: bool,
    pub media_upload_available: bool,
}

pub fn capability_catalog(scope: Option<&WhatsAppCapabilityScope>) -> Vec<WhatsAppCapability> {
    let mut capabilities = vec![
        capability(
            "runtime.hidden_webview",
            "runtime",
            Available,
            Read,
            false,
            true,
            "Hidden WebView companion is the only provider runtime and emits append-only evidence observations.",
        ),
        capability(
            "sessions.manual_state",
            "sessions",
            Available,
            Read,
            false,
            true,
            "Companion session metadata is stored without raw session secrets.",
        ),
        capability(
            "sessions.restore",
            "sessions",
            Degraded,
            SecretAccess,
            false,
            true,
            "Authorized session material can be restored only through a runtime-safe host binding.",
        ),
        capability(
            "auth.qr_link_start",
            "auth",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Link lifecycle exists, but live QR material is not emitted yet.",
        ),
        capability(
            "auth.pair_code_link_start",
            "auth",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Link lifecycle exists, but live pair-code material is not emitted yet.",
        ),
        capability(
            "sync.chats",
            "sync",
            Available,
            Read,
            false,
            true,
            "Projected conversations can be read through the provider control surface.",
        ),
        capability(
            "sync.history",
            "sync",
            Available,
            Read,
            false,
            true,
            "Projected message history can be replayed through the provider read surface.",
        ),
        capability(
            "messages.read_projection",
            "messages",
            Available,
            Read,
            false,
            true,
            "Provider message projections are available for reads.",
        ),
        capability(
            "search.messages",
            "search",
            Available,
            Read,
            false,
            true,
            "Provider message projections can be searched.",
        ),
        capability(
            "search.media",
            "search",
            Available,
            Read,
            false,
            true,
            "Projected media metadata can be searched.",
        ),
        capability(
            "messages.send_text",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Durable provider command and reconciliation exist; live execution remains blocked.",
        ),
        capability(
            "messages.reply",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Reply command and reconciliation exist; live execution remains blocked.",
        ),
        capability(
            "messages.forward",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Forward command and reconciliation exist; live execution remains blocked.",
        ),
        capability(
            "messages.edit",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Message version projection exists; live execution remains blocked.",
        ),
        capability(
            "messages.delete",
            "messages",
            Degraded,
            Destructive,
            true,
            true,
            "Message tombstone projection exists; live execution remains blocked.",
        ),
        capability(
            "messages.react",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Reaction command and reconciliation exist; live execution remains blocked.",
        ),
        capability(
            "messages.unreact",
            "messages",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Reaction removal command exists; live execution remains blocked.",
        ),
        capability(
            "media.upload_send",
            "media",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Media metadata and durable command exist; live transfer remains blocked.",
        ),
        capability(
            "media.download",
            "media",
            Degraded,
            Read,
            false,
            false,
            "Media metadata exists; live transfer remains blocked.",
        ),
        capability(
            "media.voice_send",
            "media",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Voice-note command exists; live transfer remains blocked.",
        ),
        capability(
            "conversations.join_group",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Durable command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.leave_group",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Durable command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.archive",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.unarchive",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.mute",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.unmute",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.pin",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.unpin",
            "conversations",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Dialog command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.mark_read",
            "conversations",
            Degraded,
            ProviderWrite,
            false,
            true,
            "Read-state command exists; live provider execution remains blocked.",
        ),
        capability(
            "conversations.mark_unread",
            "conversations",
            Degraded,
            ProviderWrite,
            false,
            true,
            "Unread-state command exists; live provider execution remains blocked.",
        ),
        capability(
            "status.observe",
            "status",
            Available,
            Read,
            false,
            true,
            "Status metadata evidence can be projected and replayed.",
        ),
        capability(
            "status.publish",
            "status",
            Degraded,
            ProviderWrite,
            true,
            true,
            "Durable status command exists; live execution remains blocked.",
        ),
        capability(
            "presence.observe",
            "presence",
            Available,
            Read,
            false,
            true,
            "Presence metadata evidence can be projected and replayed.",
        ),
        capability(
            "calls.observe",
            "calls",
            Available,
            Read,
            false,
            true,
            "Call metadata evidence can be projected and replayed.",
        ),
    ];

    if let Some(scope) = scope {
        apply_scope_overrides(&mut capabilities, scope);
    }
    capabilities
}

fn apply_scope_overrides(capabilities: &mut [WhatsAppCapability], scope: &WhatsAppCapabilityScope) {
    let runtime_kind = scope.runtime_kind.as_deref().unwrap_or("webview_companion");
    let lifecycle = scope.lifecycle_state.as_deref().unwrap_or("linked");
    for item in capabilities {
        match item.capability.as_str() {
            "runtime.hidden_webview" if runtime_kind != "webview_companion" => {
                block(
                    item,
                    "This account does not use the hidden WebView runtime.",
                );
                item.status = WhatsAppCapabilityState::Unsupported;
            }
            "sessions.restore" if matches!(lifecycle, "provisioning" | "link_required") => {
                block(
                    item,
                    "The account has not completed provider session linking.",
                );
            }
            "sessions.restore" if matches!(lifecycle, "revoked" | "retired") => {
                block(
                    item,
                    "The account must be relinked or is removed from runtime control.",
                );
            }
            "auth.qr_link_start" if lifecycle == "pair_code_pending" => {
                block(
                    item,
                    "Pair-code linking is already pending for this account.",
                );
            }
            "auth.pair_code_link_start" if lifecycle == "qr_pending" => {
                block(item, "QR linking is already pending for this account.");
            }
            "media.download" if !scope.media_download_available => {
                block(item, "This runtime is not live-enabled for media download.");
            }
            "media.upload_send" | "media.voice_send" if !scope.media_upload_available => {
                block(item, "This runtime is not live-enabled for media upload.");
            }
            name if is_provider_write(name) => {
                if matches!(
                    lifecycle,
                    "provisioning"
                        | "link_required"
                        | "qr_pending"
                        | "pair_code_pending"
                        | "revoked"
                        | "retired"
                ) {
                    block(
                        item,
                        "Provider commands are blocked while the account is not linked.",
                    );
                } else if !scope.live_send_available {
                    block(
                        item,
                        "This runtime is not live-enabled for provider execution.",
                    );
                }
            }
            name if is_runtime_observe(name) && lifecycle == "retired" => {
                block(item, "Removed accounts do not project runtime evidence.");
            }
            _ => {}
        }
    }
}

fn block(item: &mut WhatsAppCapability, reason: &str) {
    item.status = WhatsAppCapabilityState::Blocked;
    item.reason = reason.to_owned();
}

fn is_provider_write(name: &str) -> bool {
    matches!(
        name,
        "auth.qr_link_start"
            | "auth.pair_code_link_start"
            | "messages.send_text"
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
    )
}

fn is_runtime_observe(name: &str) -> bool {
    matches!(
        name,
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

fn capability(
    capability: &str,
    category: &str,
    status: WhatsAppCapabilityState,
    action_class: WhatsAppActionClass,
    confirmation_required: bool,
    closure_gate: bool,
    reason: &str,
) -> WhatsAppCapability {
    WhatsAppCapability {
        capability: capability.to_owned(),
        category: category.to_owned(),
        status,
        action_class,
        confirmation_required,
        closure_gate,
        reason: reason.to_owned(),
    }
}

use WhatsAppActionClass::{Destructive, ProviderWrite, Read, SecretAccess};
use WhatsAppCapabilityState::{Available, Degraded};
