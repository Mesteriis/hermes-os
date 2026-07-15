use chrono::Utc;
use sha2::{Digest, Sha256};

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub(super) fn new_version_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgver_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("ver_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

pub(super) fn new_tombstone_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgtomb_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("tomb_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}
