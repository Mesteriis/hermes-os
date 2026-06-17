use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_extended_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
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
        "media.gallery",
        cat_media,
        TelegramCapabilityState::Blocked,
        TelegramActionClass::Read,
        "Media gallery and search require dedicated projection and UI.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "media.preview",
        cat_media,
        TelegramCapabilityState::Degraded,
        TelegramActionClass::Read,
        "Shared Communication attachment preview may work; no Telegram-specific preview surface.",
        false,
        false,
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
        "realtime.message_created",
        cat_rt,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "telegram.message.created realtime events are emitted for fixture ingest and manual sends.",
        false,
        false,
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
}
