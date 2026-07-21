//! Minimal IMAP read path adapter boundary for ADR-0239.

pub const PACKAGE: &str = "hermes-mail-imap";

pub const MAX_ATTEMPTS: u8 = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct ImapMessage {
    pub uid: u32,
    pub subject: String,
    pub snippet: String,
    pub has_plain_text: bool,
}

#[derive(Clone, Debug)]
pub struct ImapSyncResult {
    pub messages: Vec<ImapMessage>,
    pub attempts: u8,
    pub window: u32,
    pub has_more: bool,
}

fn deterministic_uid(host: &str, index: u32) -> u32 {
    let mut hash = 1469598103934665603u64;
    for byte in host.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    let mixed = hash.wrapping_add(u64::from(index));
    (mixed & 0xFFFF_FFFF) as u32
}

pub fn synthetic_inbox(host: &str, window: u32, windows: u32) -> ImapSyncResult {
    let size = window.saturating_mul(windows);
    let mut index = 0;
    let mut messages = Vec::new();
    while index < size {
        let uid = deterministic_uid(host, index);
        messages.push(ImapMessage {
            uid,
            subject: format!("Mail {index}"),
            snippet: format!("{} #{}", host, index),
            has_plain_text: index % 2 == 0,
        });
        index += 1;
    }
    ImapSyncResult {
        messages,
        attempts: 1,
        window,
        has_more: false,
    }
}

pub fn supports_read_only_sync(window: u32) -> bool {
    window > 0 && window <= 500
}
