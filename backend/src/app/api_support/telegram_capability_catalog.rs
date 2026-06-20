use super::telegram_capabilities::TelegramOperationCapability;

#[cfg(test)]
#[path = "telegram_capability_catalog_tests.rs"]
mod telegram_capability_catalog_tests;

pub(super) fn telegram_capability_rows(qr_ready: bool) -> Vec<TelegramOperationCapability> {
    let mut capabilities = Vec::new();
    super::telegram_capability_catalog_foundation::push_foundation_capabilities(
        &mut capabilities,
        qr_ready,
    );
    super::telegram_capability_catalog_messages::push_message_capabilities(
        &mut capabilities,
        qr_ready,
    );
    super::telegram_capability_catalog_extended::push_extended_capabilities(
        &mut capabilities,
        qr_ready,
    );
    capabilities
}
