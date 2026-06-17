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
    assert_eq!(mark_read.status, "degraded");
    assert_eq!(mark_read.action_class, "provider_write");

    let mark_unread = capability(&capabilities, "dialogs.mark_unread");
    assert_eq!(mark_unread.status, "degraded");
    assert_eq!(mark_unread.action_class, "provider_write");
}
