use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn telegram_capability_rows(qr_ready: bool) -> Vec<TelegramOperationCapability> {
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
        "runtime.stop",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Account-scoped runtime actor stop is available and preserves local evidence.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.restart",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Account-scoped runtime actor restart is available and preserves local evidence.",
        true,
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
        "dialogs.pin",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::LocalWrite,
        "Local projected pin/unpin is available; provider-synced parity still requires durable outbox execution.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.archive",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Local projection is updated immediately; active TDLib actors execute queued addChatToList archive/unarchive commands, while provider-observed folder reconciliation remains incomplete.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mute",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Local projection is updated immediately; active TDLib actors execute queued setChatNotificationSettings mute/unmute commands, while provider-observed notification reconciliation remains incomplete.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.unread_counters",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::Read,
        "Local projected unread counters are available; provider-observed unread reconciliation is still missing.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mark_read",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Local projection is updated immediately; active TDLib actors execute queued provider manual unread toggles for read/unread parity.",
        false,
        false,
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
        "messages.get_versions",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Observed Telegram edit versions are available through the lifecycle history endpoints.",
        false,
        false,
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
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider-side media upload/send is available through local attachment import and durable outbox."
        } else {
            "Media upload/send requires TDLib QR runtime, local attachment import and durable outbox."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
            "messages.edit",
            cat_msg_write,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::ProviderWrite,
            if qr_ready {
                "Append-only edit history and TDLib provider edit execution are available through the command executor."
            } else {
                "Append-only edit history is available locally; provider edit execution requires TDLib QR runtime."
            },
            true,
            true,
        ));
    capabilities.push(TelegramOperationCapability::new(
            "messages.delete",
            cat_msg_write,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::Destructive,
            if qr_ready {
                "Tombstone recording and TDLib provider delete execution are available through the command executor."
            } else {
                "Tombstone recording is available locally; provider delete execution requires TDLib QR runtime."
            },
            true,
            true,
        ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.restore_visibility",
        cat_msg_write,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Local visibility restore writes tombstone history and command audit evidence.",
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
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "TDLib reply send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.forward",
        cat_reply,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "TDLib forward send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
            "messages.pin",
            cat_reply,
            if qr_ready {
                TelegramCapabilityState::Available
            } else {
                TelegramCapabilityState::Degraded
            },
            TelegramActionClass::ProviderWrite,
            if qr_ready {
                "Local pin projection and TDLib provider pin execution are available through the command executor."
            } else {
                "Local pin projection is available; provider pin execution requires TDLib QR runtime."
            },
            true,
            true,
        ));

    // ── reactions ──
    let cat_reactions = "reactions";
    capabilities.push(TelegramOperationCapability::new(
            "reactions.add", cat_reactions,
            if qr_ready { TelegramCapabilityState::Available } else { TelegramCapabilityState::Degraded },
            TelegramActionClass::ProviderWrite,
            if qr_ready {
                "Local reaction projection and TDLib provider reaction execution are available through the command executor."
            } else {
                "Local reaction projection is available; provider reaction execution requires TDLib QR runtime."
            },
            false, true,
        ));
    capabilities.push(TelegramOperationCapability::new(
            "reactions.remove", cat_reactions,
            if qr_ready { TelegramCapabilityState::Available } else { TelegramCapabilityState::Degraded },
            TelegramActionClass::ProviderWrite,
            if qr_ready {
                "Local reaction removal and TDLib provider reaction execution are available through the command executor."
            } else {
                "Local reaction removal is available; provider reaction execution requires TDLib QR runtime."
            },
            false, true,
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

    // ── participants ──
    let cat_participants = "participants";
    capabilities.push(TelegramOperationCapability::new(
        "participants.sync",
        cat_participants,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::Read,
        if qr_ready {
            "TDLib provider member roster sync is available for supergroups/channels."
        } else {
            "Provider member roster sync requires TDLib QR runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.join",
        cat_participants,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "TDLib chat join is available through the durable provider-write outbox."
        } else {
            "Chat join requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.leave",
        cat_participants,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::Destructive,
        if qr_ready {
            "TDLib chat leave is available through the durable provider-write outbox."
        } else {
            "Chat leave requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
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

    super::telegram_capability_catalog_extended::push_extended_capabilities(
        &mut capabilities,
        qr_ready,
    );
    capabilities
}
