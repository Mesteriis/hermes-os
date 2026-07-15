use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) fn stable_whatsapp_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}

pub(super) fn reaction(account_id: &str, message_id: &str, actor_id: &str, value: &str) -> String {
    stable_whatsapp_id(
        "reaction:v5:whatsapp_web",
        &[account_id, message_id, actor_id, value],
    )
}
pub(super) fn status_message(account_id: &str, status_id: &str) -> String {
    stable_whatsapp_id("message:v5:whatsapp_web", &[account_id, status_id])
}
pub(super) fn call(account_id: &str, call_id: &str) -> String {
    stable_whatsapp_id("call:v5:whatsapp_web", &[account_id, call_id])
}
pub(super) fn channel(account_id: &str) -> String {
    stable_whatsapp_id("channel:v5:whatsapp_web", &[account_id])
}
pub(super) fn conversation(account_id: &str, chat_id: &str) -> String {
    stable_whatsapp_id("conversation:v5:whatsapp_web", &[account_id, chat_id])
}
pub(super) fn status_feed_conversation(account_id: &str) -> String {
    format!("whatsapp_status_feed:{account_id}")
}
pub(super) fn identity(account_id: &str, kind: &str, identity_id: &str) -> String {
    stable_whatsapp_id("identity:v5:whatsapp_web", &[account_id, kind, identity_id])
}
pub(super) fn participant(conversation_id: &str, member_id: &str) -> String {
    stable_whatsapp_id("participant:v5:whatsapp_web", &[conversation_id, member_id])
}
pub(super) fn message_version(event_id: &str) -> String {
    stable_whatsapp_id("message_version:v5:whatsapp_web", &[event_id])
}
pub(super) fn message_tombstone(event_id: &str) -> String {
    stable_whatsapp_id("message_tombstone:v5:whatsapp_web", &[event_id])
}
pub(super) fn message_ref(
    account_id: &str,
    kind: &str,
    source_id: &str,
    target_id: Option<&str>,
) -> String {
    stable_whatsapp_id(
        "message_ref:v5:whatsapp_web",
        &[account_id, kind, source_id, target_id.unwrap_or("")],
    )
}
pub(super) fn runtime_event_raw(account_id: &str, event_id: &str) -> String {
    stable_whatsapp_id(
        "raw:v5:whatsapp_web",
        &[account_id, "whatsapp_web_runtime_event", event_id],
    )
}

pub(super) fn fixture_event(scope: &str, subject: &str, now: DateTime<Utc>) -> String {
    let seq = FIXTURE_EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_fixture_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}
