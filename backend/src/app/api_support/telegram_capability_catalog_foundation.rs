use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_foundation_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
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
        "runtime.bot_live",
        cat_runtime,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Bot API runtime is deferred to the separate Bot Runtime initiative.",
        false,
        true,
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

    let cat_auth = "authorization";
    push_auth_capabilities(capabilities, cat_auth, qr_ready);
    push_session_and_sync_capabilities(capabilities, qr_ready);
    push_dialog_capabilities(capabilities, qr_ready);
}

fn push_auth_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    cat_auth: &str,
    qr_ready: bool,
) {
    capabilities.push(TelegramOperationCapability::new(
        "auth.qr_start",
        cat_auth,
        qr_state(qr_ready),
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
        qr_state(qr_ready),
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
        qr_state(qr_ready),
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
        qr_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "QR login cancellation is available."
        } else {
            "QR cancel requires native TDLib and app credentials."
        },
        false,
        false,
    ));
}

fn push_session_and_sync_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_session = "session";
    capabilities.push(TelegramOperationCapability::new(
        "session.import",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "Session import is deferred to a separate encrypted session-portability initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "session.export",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Export,
        "Session export is deferred to a separate encrypted session-portability initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.configure",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "Proxy profiles are deferred to separate MTProxy/SOCKS5 runtime work.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.mtproxy",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "MTProxy support is deferred to a separate proxy initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.socks5",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "SOCKS5 support is deferred to a separate proxy initiative.",
        false,
        true,
    ));

    let cat_sync = "sync";
    capabilities.push(TelegramOperationCapability::new(
        "sync.chats",
        cat_sync,
        sync_state(qr_ready),
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
        sync_state(qr_ready),
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
        sync_state(qr_ready),
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
        sync_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "Full history sync is available."
        } else {
            "Full history sync limited to fixture runtime."
        },
        false,
        false,
    ));
}

fn push_dialog_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
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
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog pin/unpin uses the durable provider-write outbox and provider-observed TDLib chat-position reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.archive",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog archive/unarchive uses the durable provider-write outbox and provider-observed TDLib chat-position reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mute",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog mute/unmute uses the durable provider-write outbox and provider-observed TDLib notification-settings reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.unread_counters",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Read,
        "Projected unread and mention counters are available and provider-observed TDLib chat-state updates reconcile them into shared chat metadata.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mark_read",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Mark-read uses the durable provider-write outbox and provider-observed TDLib read-inbox reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.folder_add",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Adding a chat to a Telegram folder uses the durable provider-write outbox and TDLib chat-position reconciliation for the target folder.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.folder_remove",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Removing a chat from a Telegram folder uses the durable provider-write outbox plus TDLib folder-edit reconciliation when the provider confirms removal.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.folder_reassign",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Reassigning Telegram folder membership computes a durable add/remove command set from the current TDLib folder projection and queues those provider-write commands atomically from one API action.",
        false,
        false,
    ));
}

fn qr_state(qr_ready: bool) -> TelegramCapabilityState {
    if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Blocked
    }
}

fn sync_state(qr_ready: bool) -> TelegramCapabilityState {
    if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Degraded
    }
}

fn provider_or_unsupported(qr_ready: bool) -> TelegramCapabilityState {
    if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Unsupported
    }
}
