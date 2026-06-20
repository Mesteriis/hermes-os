use super::super::telegram_capabilities::TelegramCapabilityState;
use super::super::telegram_capability_catalog::telegram_capability_rows;

fn capability<'a>(
    capabilities: &'a [super::TelegramOperationCapability],
    operation: &str,
) -> &'a super::TelegramOperationCapability {
    capabilities
        .iter()
        .find(|item| item.operation == operation)
        .expect("capability exists")
}

#[test]
fn deferred_telegram_initiatives_are_api_visible_planned_capabilities() {
    assert_eq!(TelegramCapabilityState::Planned.as_str(), "planned");

    let capabilities = telegram_capability_rows(false);
    for operation in [
        "runtime.bot_live",
        "voice.record",
        "voice.send",
        "video.record",
        "calls.live_control",
        "session.import",
        "session.export",
        "proxy.mtproxy",
        "proxy.socks5",
        "ai.summary",
        "ai.translation",
        "ai.bilingual_reply",
        "ai.review_flows",
    ] {
        let capability = capability(&capabilities, operation);
        assert_eq!(capability.status, "planned", "{operation}");
    }
}

#[test]
fn qr_ready_dialog_capabilities_reflect_provider_write_reconciliation() {
    let capabilities = telegram_capability_rows(true);

    let pin = capability(&capabilities, "dialogs.pin");
    assert_eq!(pin.status, "available");
    assert_eq!(pin.action_class, "provider_write");

    let archive = capability(&capabilities, "dialogs.archive");
    assert_eq!(archive.status, "available");
    assert_eq!(archive.action_class, "provider_write");

    let mute = capability(&capabilities, "dialogs.mute");
    assert_eq!(mute.status, "available");
    assert_eq!(mute.action_class, "provider_write");

    let unread_counters = capability(&capabilities, "dialogs.unread_counters");
    assert_eq!(unread_counters.status, "available");
    assert_eq!(unread_counters.action_class, "read");

    let mark_read = capability(&capabilities, "dialogs.mark_read");
    assert_eq!(mark_read.status, "available");
    assert_eq!(mark_read.action_class, "provider_write");

    let mark_unread = capability(&capabilities, "dialogs.mark_unread");
    assert_eq!(mark_unread.status, "available");
    assert_eq!(mark_unread.action_class, "provider_write");

    let reaction_sync = capability(&capabilities, "reactions.sync");
    assert_eq!(reaction_sync.status, "available");
    assert_eq!(reaction_sync.action_class, "read");
}

#[test]
fn qr_ready_search_and_media_capabilities_reflect_projection_backed_provider_refresh() {
    let capabilities = telegram_capability_rows(true);

    for operation in [
        "search.provider",
        "search.media",
        "media.gallery",
        "media.preview",
        "voice.playback",
    ] {
        let capability = capability(&capabilities, operation);
        assert_eq!(capability.status, "available", "{operation}");
        assert_eq!(capability.action_class, "read", "{operation}");
    }
}
