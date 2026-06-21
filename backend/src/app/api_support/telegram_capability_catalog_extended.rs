use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_extended_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    // ── dialogs ──
    let cat_dialogs = "dialogs";
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mark_unread",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Mark-unread relies on durable provider-write commands and TDLib unread-state reconciliation.",
        false,
        false,
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
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Media gallery is backed by projected Telegram attachment metadata plus query-backed media search results.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "media.preview",
        cat_media,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Telegram media preview uses the shared Communication attachment preview boundary and local downloaded media paths.",
        false,
        false,
    ));

    // ── voice / calls ──
    let cat_voice = "voice_calls";
    capabilities.push(TelegramOperationCapability::new(
        "voice.playback",
        cat_voice,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Projected Telegram voice/audio attachments play from local downloaded media through the shared media viewer.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.record_send",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice recording/send is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.record",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice recording is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.send",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice send is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "video.record",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Video recording is deferred to a separate Voice/Calls initiative.",
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
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Live call control is deferred to the separate Calls initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "calls.transcription_live",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Live call transcription is deferred to the separate Calls/AI initiatives.",
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
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Read,
        "Provider-side TDLib search refreshes provider results into projection before returning UI-visible results.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "search.media",
        cat_search,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Media search reads projected Telegram attachment metadata and attempts provider refresh when account and query are available.",
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
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific summary is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.translation",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific translation is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.bilingual_reply",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific bilingual reply is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.review_flows",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific AI review flows are deferred to the separate AI Layer initiative.",
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
