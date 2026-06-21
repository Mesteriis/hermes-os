use chrono::Utc;
use sha2::{Digest, Sha256};

use super::super::identifiers::safe_path_segment;

pub(in crate::integrations::telegram::tdjson) fn new_setup_id(account_id: &str) -> String {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(timestamp.to_string().as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!(
        "telegram-qr-{}-{}",
        safe_path_segment(account_id),
        &digest[..16]
    )
}

pub(in crate::integrations::telegram::tdjson) fn short_thread_suffix(account_id: &str) -> String {
    safe_path_segment(account_id).chars().take(32).collect()
}
