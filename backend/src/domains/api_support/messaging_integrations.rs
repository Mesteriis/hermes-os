use super::*;
use crate::domains::mail::core::ProviderAccount;

// ---------------------------------------------------------------------------

/// Capability states per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCapabilityState {
    Available,
    Blocked,
    Degraded,
    Unsupported,
}

impl TelegramCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Action classes per ADR-0052.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl TelegramActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::Export => "export",
            Self::SecretAccess => "secret_access",
            Self::Automation => "automation",
        }
    }
}

/// A single operation capability entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramOperationCapability {
    pub operation: String,
    pub category: String,
    pub status: String,
    pub action_class: String,
    pub reason: String,
    pub confirmation_required: bool,
    pub closure_gate: bool,
}

impl TelegramOperationCapability {
    fn new(
        operation: &str,
        category: &str,
        state: TelegramCapabilityState,
        action_class: TelegramActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            operation: operation.to_owned(),
            category: category.to_owned(),
            status: state.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            reason: reason.to_owned(),
            confirmation_required,
            closure_gate,
        }
    }
}

/// Detailed per-operation Telegram capability response per ADR-0091.
#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) account_scope: Option<TelegramCapabilityAccountScope>,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) bot_runtime_available: bool,
    pub(crate) capabilities: Vec<TelegramOperationCapability>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramCapabilityAccountScope {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub lifecycle_state: String,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        Self::build(config, None)
    }

    pub(crate) fn current_for_account(config: &AppConfig, account: &ProviderAccount) -> Self {
        Self::build(config, Some(account))
    }

    fn build(config: &AppConfig, account: Option<&ProviderAccount>) -> Self {
        let app_creds = config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_ok = tdjson::runtime_available(config.tdjson_path());
        let qr_ready = app_creds && tdjson_ok;
        let bot_ok = false; // Bot API runtime not implemented per ADR-0091
        let account_scope = account.map(TelegramCapabilityAccountScope::from_account);

        let mut capabilities = Vec::new();

        // ── account lifecycle ──
        let cat_account = "account";
        capabilities.push(TelegramOperationCapability::new(
            "account.create_user",
            cat_account,
            TelegramCapabilityState::Available,
            TelegramActionClass::SecretAccess,
            "User account setup with host-vault secret binding is available.",
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "account.create_bot",
            cat_account,
            TelegramCapabilityState::Available,
            TelegramActionClass::SecretAccess,
            "Bot account metadata and bot-token secret binding are available.",
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "account.list",
            cat_account,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Account list with lifecycle state and runtime mode is available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "account.logout",
            cat_account,
            TelegramCapabilityState::Available,
            TelegramActionClass::Destructive,
            "Account logout stops runtime actor and marks lifecycle state.",
            true,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "account.remove",
            cat_account,
            TelegramCapabilityState::Available,
            TelegramActionClass::Destructive,
            "Account removal preserves local evidence, marks account removed and stops runtime.",
            true,
            false,
        ));

        // ── runtime ──
        let cat_runtime = "runtime";
        capabilities.push(TelegramOperationCapability::new(
            "runtime.fixture",
            cat_runtime,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Fixture runtime is available for CI and local smoke validation.",
            false,
            true,
        ));
        let tdlib_live = if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        };
        let tdlib_reason = if qr_ready {
            "TDLib QR login runtime is configured for local development."
        } else {
            "Live TDLib sessions require native TDLib JSON runtime and Telegram app credentials."
        };
        capabilities.push(TelegramOperationCapability::new(
            "runtime.tdlib_live",
            cat_runtime,
            tdlib_live,
            TelegramActionClass::Read,
            tdlib_reason,
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "runtime.bot_live", cat_runtime,
            TelegramCapabilityState::Blocked, TelegramActionClass::Read,
            "Live bot sends require the Bot API runtime adapter and account-scoped bot token resolution.",
            false, true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "runtime.status",
            cat_runtime,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Account-scoped runtime status is available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "runtime.health_details",
            cat_runtime,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Detailed TDLib/native dependency health diagnostics are not yet implemented.",
            false,
            false,
        ));

        // ── authorization ──
        let cat_auth = "authorization";
        capabilities.push(TelegramOperationCapability::new(
            "auth.qr_start",
            cat_auth,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Blocked
            },
            TelegramActionClass::SecretAccess,
            if qr_ready {
                "QR login start is available."
            } else {
                "QR login requires native TDLib and app credentials."
            },
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "auth.qr_status",
            cat_auth,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Blocked
            },
            TelegramActionClass::Read,
            if qr_ready {
                "QR status polling is available."
            } else {
                "QR status requires native TDLib and app credentials."
            },
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "auth.qr_password",
            cat_auth,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Blocked
            },
            TelegramActionClass::SecretAccess,
            if qr_ready {
                "2FA password submission is available."
            } else {
                "2FA submission requires native TDLib and app credentials."
            },
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "auth.qr_cancel",
            cat_auth,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Blocked
            },
            TelegramActionClass::Read,
            if qr_ready {
                "QR login cancellation is available."
            } else {
                "QR cancel requires native TDLib and app credentials."
            },
            false,
            false,
        ));

        // ── session / proxy ──
        let cat_session = "session";
        capabilities.push(TelegramOperationCapability::new(
            "session.import",
            cat_session,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::SecretAccess,
            "Session import requires encrypted bundle contract and host-vault unlock.",
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "session.export",
            cat_session,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::Export,
            "Session export requires encrypted bundle contract and host-vault unlock.",
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "proxy.configure",
            cat_session,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::SecretAccess,
            "Proxy profiles require SOCKS5/MTProto secret storage and runtime restart.",
            false,
            true,
        ));

        // ── sync ──
        let cat_sync = "sync";
        capabilities.push(TelegramOperationCapability::new(
            "sync.chats",
            cat_sync,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Read,
            if qr_ready {
                "Chat sync through TDLib runtime is available."
            } else {
                "Chat sync limited to fixture runtime."
            },
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "sync.history_latest",
            cat_sync,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Read,
            if qr_ready {
                "Latest history sync is available."
            } else {
                "History sync limited to fixture runtime."
            },
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "sync.history_older",
            cat_sync,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Read,
            if qr_ready {
                "Older history pagination is available."
            } else {
                "Older history pagination limited to fixture runtime."
            },
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "sync.history_full",
            cat_sync,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Read,
            if qr_ready {
                "Full history sync is available."
            } else {
                "Full history sync limited to fixture runtime."
            },
            false,
            false,
        ));

        // ── dialogs / chats ──
        let cat_dialogs = "dialogs";
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.list",
            cat_dialogs,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Projected chat list is available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.pin", cat_dialogs,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local projected pin/unpin is available; provider-synced parity still requires durable outbox execution.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.archive", cat_dialogs,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local projected archive/unarchive is available; provider-synced parity still requires durable outbox execution.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.mute", cat_dialogs,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local projected mute/unmute is available; provider-synced parity still requires durable outbox execution.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.unread_counters", cat_dialogs,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::Read,
            "Local projected unread counters are available; provider-synced unread state is still missing.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "dialogs.mark_read", cat_dialogs,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local projected mark read/unread is available; provider-synced read state still requires durable outbox execution.",
            false, false,
        ));

        // ── messages: read ──
        let cat_msg_read = "messages:read";
        capabilities.push(TelegramOperationCapability::new(
            "messages.list",
            cat_msg_read,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Projected message list is available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.get_versions", cat_msg_read,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "Observed Telegram edit versions are available through the lifecycle history endpoints.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.get_raw_evidence",
            cat_msg_read,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Sanitized raw provider evidence view is available.",
            false,
            false,
        ));

        // ── messages: write ──
        let cat_msg_write = "messages:write";
        capabilities.push(TelegramOperationCapability::new(
            "messages.send_text",
            cat_msg_write,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::ProviderWrite,
            if qr_ready {
                "Manual text send through TDLib QR runtime is available."
            } else {
                "Manual text send limited to fixture runtime."
            },
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.send_media",
            cat_msg_write,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Media upload/send requires durable outbox model and attachment upload pipeline.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.edit",
            cat_msg_write,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Edit requires message_versions table, provider command model and audit boundary.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.delete",
            cat_msg_write,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Destructive,
            "Delete requires tombstones table, provider command model and audit boundary.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.restore_visibility",
            cat_msg_write,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::LocalWrite,
            "Restore visibility requires tombstones table and local visibility state model.",
            true,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.mark_read", cat_msg_write,
            TelegramCapabilityState::Blocked, TelegramActionClass::ProviderWrite,
            "Provider-synced mark read/unread still requires durable command execution; only local dialog-level state exists.",
            true, true,
        ));

        // ── replies / forwards ──
        let cat_reply = "replies_forwards";
        capabilities.push(TelegramOperationCapability::new(
            "messages.reply",
            cat_reply,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Reply requires reply_target projection and provider command model.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.forward",
            cat_reply,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Forward requires forward_attribution projection and provider command model.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "messages.pin",
            cat_reply,
            TelegramCapabilityState::Degraded,
            TelegramActionClass::LocalWrite,
            "Local message pin/unpin projection is available; provider-side pin sync still requires durable outbox parity.",
            true,
            false,
        ));

        // ── reactions ──
        let cat_reactions = "reactions";
        capabilities.push(TelegramOperationCapability::new(
            "reactions.add", cat_reactions,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local reaction projection is available; provider-side reaction execution still requires durable outbox parity.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "reactions.remove", cat_reactions,
            if qr_ready { TelegramCapabilityState::Degraded } else { TelegramCapabilityState::Unsupported },
            TelegramActionClass::LocalWrite,
            "Local reaction removal is available; provider-side reaction execution still requires durable outbox parity.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "reactions.sync",
            cat_reactions,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Reaction sync requires parser, projection and realtime contract.",
            false,
            false,
        ));

        // ── topics ──
        let cat_topics = "topics";
        capabilities.push(TelegramOperationCapability::new(
            "topics.list",
            cat_topics,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::Read,
            "Topic projection and topic-scoped timeline not yet implemented.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "topics.create",
            cat_topics,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::ProviderWrite,
            "Topic create requires forum/supergroup topic projection model.",
            true,
            true,
        ));

        // ── media ──
        let cat_media = "media";
        capabilities.push(TelegramOperationCapability::new(
            "media.download",
            cat_media,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Read,
            if qr_ready {
                "TDLib media download is available."
            } else {
                "Media download limited to fixture runtime (fails closed)."
            },
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "media.upload_send",
            cat_media,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Media upload/send requires durable outbox and attachment upload pipeline.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "media.gallery",
            cat_media,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Media gallery and search require dedicated projection and UI.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "media.preview", cat_media,
            TelegramCapabilityState::Degraded, TelegramActionClass::Read,
            "Shared Communication attachment preview may work; no Telegram-specific preview surface.",
            false, false,
        ));

        // ── voice / calls ──
        let cat_voice = "voice_calls";
        capabilities.push(TelegramOperationCapability::new(
            "voice.playback",
            cat_voice,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Voice message playback requires dedicated player UI and local blob access.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "voice.record_send",
            cat_voice,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Voice recording/send requires desktop media permission boundary.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "calls.metadata",
            cat_voice,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Call metadata and fixture transcript storage are available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "calls.live_control",
            cat_voice,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::ProviderWrite,
            "Live call control requires native desktop permission ADR.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "calls.transcription_live",
            cat_voice,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Live transcription requires STT provider adapter and model configuration.",
            false,
            true,
        ));

        // ── search ──
        let cat_search = "search";
        capabilities.push(TelegramOperationCapability::new(
            "search.local_messages",
            cat_search,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Local thread search/filter and shared Communication search are available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "search.local_dialogs",
            cat_search,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Local chat title filter is available.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "search.provider",
            cat_search,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Provider-side TDLib search requires dedicated API and UI.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "search.media",
            cat_search,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Media search requires dedicated media projection and gallery.",
            false,
            false,
        ));

        // ── realtime ──
        let cat_rt = "realtime";
        capabilities.push(TelegramOperationCapability::new(
            "realtime.generic_transport",
            cat_rt,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "Generic WebSocket/SSE/long-poll transports exist at platform level.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "realtime.message_created", cat_rt,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "telegram.message.created realtime events are emitted for fixture ingest and manual sends.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "realtime.message_updated", cat_rt,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "telegram.message.updated realtime events are emitted when lifecycle edit records are created.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "realtime.message_deleted",
            cat_rt,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "telegram.message.deleted realtime events are emitted when tombstones are recorded.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "realtime.reaction_changed", cat_rt,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "telegram.reaction.changed realtime events are emitted for local reaction add/remove actions.",
            false, false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "realtime.sync_progress",
            cat_rt,
            TelegramCapabilityState::Available,
            TelegramActionClass::Read,
            "telegram.sync progress events are emitted for chat and history sync requests.",
            false,
            false,
        ));

        // ── automation ──
        let cat_auto = "automation";
        capabilities.push(TelegramOperationCapability::new(
            "automation.dry_run",
            cat_auto,
            TelegramCapabilityState::Available,
            TelegramActionClass::Automation,
            "Policy/template validation and audited dry-run records are available.",
            false,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "automation.live_send", cat_auto,
            TelegramCapabilityState::Blocked, TelegramActionClass::Automation,
            "Live automated sends blocked until live runtime passes the same policy evaluator and audit contract.",
            true, true,
        ));

        // ── ai ──
        let cat_ai = "ai";
        capabilities.push(TelegramOperationCapability::new(
            "ai.summary",
            cat_ai,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Telegram-specific summary API/UI not yet implemented.",
            false,
            false,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "ai.translation",
            cat_ai,
            TelegramCapabilityState::Blocked,
            TelegramActionClass::Read,
            "Telegram-specific translation route/UI not yet implemented.",
            false,
            false,
        ));

        // ── export ──
        let cat_export = "export";
        capabilities.push(TelegramOperationCapability::new(
            "export.chat",
            cat_export,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::Export,
            "Chat export requires scope selection, audit and manifest contract.",
            true,
            true,
        ));
        capabilities.push(TelegramOperationCapability::new(
            "export.markdown",
            cat_export,
            TelegramCapabilityState::Unsupported,
            TelegramActionClass::Export,
            "Markdown export requires message-order, sender, time and attachment referencing.",
            true,
            true,
        ));

        let mut response = Self {
            version: "2.0",
            runtime_mode: if let Some(scope) = account_scope.as_ref() {
                match scope.runtime_kind.as_str() {
                    "tdlib_qr_authorized" => "tdlib_qr_authorized",
                    "live_blocked" => "live_blocked",
                    "fixture" => "fixture",
                    _ => "unknown",
                }
            } else if qr_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            account_scope,
            telegram_app_credentials_configured: app_creds,
            tdjson_runtime_available: tdjson_ok,
            qr_login_ready: qr_ready,
            bot_runtime_available: bot_ok,
            capabilities,
            unsupported_features: vec![
                "video_calls",
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
                "session_import_export",
                "proxy_profiles",
                "forum_topics",
                "chat_export",
                "bot_live_runtime",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let provider_kind = scope.provider_kind.as_str();
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let is_bot = provider_kind == "telegram_bot";
        let is_removed = lifecycle_state == "removed";
        let is_logged_out = lifecycle_state == "logged_out";

        for capability in &mut self.capabilities {
            match capability.operation.as_str() {
                "auth.qr_start" | "auth.qr_status" | "auth.qr_password" | "auth.qr_cancel"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use TDLib QR authorization.".to_owned();
                }
                "runtime.tdlib_live" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user runtime.".to_owned();
                }
                "runtime.bot_live" if !is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "User accounts do not use the Bot API runtime.".to_owned();
                }
                "messages.send_text" if is_bot && runtime_kind == "fixture" => {
                    capability.status = TelegramCapabilityState::Degraded.as_str().to_owned();
                    capability.reason = "Fixture bot accounts can validate local command flow, but the live Bot API runtime is still missing.".to_owned();
                }
                _ => {}
            }

            if is_removed {
                if matches!(
                    capability.action_class.as_str(),
                    "local_write"
                        | "provider_write"
                        | "destructive"
                        | "secret_access"
                        | "automation"
                ) || capability.operation.starts_with("sync.")
                    || capability.operation.starts_with("runtime.")
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` is removed; this operation is no longer available.",
                        scope.account_id
                    );
                }
            } else if is_logged_out
                && (capability.operation.starts_with("sync.")
                    || capability.operation == "messages.send_text"
                    || capability.operation == "media.download")
            {
                capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                capability.reason = format!(
                    "Account `{}` is logged out; re-authorize the runtime before using this operation.",
                    scope.account_id
                );
            }
        }
    }
}

impl TelegramCapabilityAccountScope {
    fn from_account(account: &ProviderAccount) -> Self {
        Self {
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            runtime_kind: account
                .config
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("unknown")
                .to_owned(),
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("active")
                .to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// WhatsApp capability model (unchanged)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current() -> Self {
        Self {
            version: "1.0",
            runtime_mode: "fixture",
            capabilities: vec![
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_runtime",
                    "Fixture WhatsApp Web accounts, session metadata and message projection are available for CI and local smoke validation.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_manual_session_state",
                    "Manual companion session metadata is stored without session secrets or pairing material in PostgreSQL.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_ingestion",
                    "Fixture WhatsApp Web messages preserve append-only raw provenance and project into canonical communication messages.",
                    true,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_runtime",
                    "Live WhatsApp Web requires a user-visible desktop runtime, explicit session lifecycle and smoke validation.",
                    false,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_send",
                    "Live outbound sends require a WhatsApp-specific policy, audit and visible runtime contract.",
                    false,
                ),
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "reverse_engineered_protocol_runtime",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "whatsapp_business_cloud_as_personal_provider",
            ],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) status: &'static str,
    pub(crate) closure_gate: bool,
    pub(crate) reason: &'static str,
}

impl WhatsappCapabilityStatus {
    fn available(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "available",
            closure_gate,
            reason,
        }
    }

    fn blocked(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "blocked",
            closure_gate,
            reason,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TelegramListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatListResponse {
    pub(crate) items: Vec<TelegramChat>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMessageListResponse {
    pub(crate) items: Vec<TelegramMessage>,
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
    pub(crate) sender_id: String,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) command_id: Option<String>,
}
