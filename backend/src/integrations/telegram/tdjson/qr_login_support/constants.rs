use std::time::Duration;

pub(in crate::integrations::telegram::tdjson) const QR_FIRST_LINK_TIMEOUT: Duration =
    Duration::from_secs(20);
pub(in crate::integrations::telegram::tdjson) const QR_SESSION_LIFETIME: Duration =
    Duration::from_secs(10 * 60);
pub(in crate::integrations::telegram::tdjson) const QR_CANCEL_WAIT_TIMEOUT: Duration =
    Duration::from_secs(5);
pub(in crate::integrations::telegram::tdjson) const QR_GET_ME_TIMEOUT: Duration =
    Duration::from_secs(5);
pub(in crate::integrations::telegram::tdjson) const QR_POLL_AFTER_MS: u64 = 2_000;
