use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::errors::EmailProviderNetworkError;

pub(super) fn trim_base_url(base_url: String) -> String {
    base_url.trim().trim_end_matches('/').to_owned()
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), EmailProviderNetworkError> {
    if value.trim().is_empty() {
        return Err(EmailProviderNetworkError::InvalidProviderRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}

pub(super) fn parse_gmail_internal_date(
    internal_date: Option<&str>,
) -> Result<Option<DateTime<Utc>>, EmailProviderNetworkError> {
    let Some(internal_date) = internal_date else {
        return Ok(None);
    };
    let millis = internal_date.parse::<i64>().map_err(|_| {
        EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "expected epoch milliseconds",
        }
    })?;

    Utc.timestamp_millis_opt(millis)
        .single()
        .ok_or(EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "timestamp is out of range",
        })
        .map(Some)
}

pub(super) fn select_latest_history_id(
    current: Option<String>,
    candidate: Option<&str>,
) -> Option<String> {
    let Some(candidate) = candidate else {
        return current;
    };
    let Some(current) = current else {
        return Some(candidate.to_owned());
    };

    let current_number = current.parse::<u64>();
    let candidate_number = candidate.parse::<u64>();
    match (current_number, candidate_number) {
        (Ok(current_number), Ok(candidate_number)) if current_number >= candidate_number => {
            Some(current)
        }
        _ => Some(candidate.to_owned()),
    }
}

pub(super) fn gmail_message_list_checkpoint(
    history_id: Option<String>,
    next_page_token: Option<String>,
) -> Option<Value> {
    gmail_checkpoint(history_id, next_page_token, Some("messages"), None)
}

pub(super) fn gmail_history_checkpoint(
    start_history_id: &str,
    history_id: Option<String>,
    next_page_token: Option<String>,
) -> Option<Value> {
    gmail_checkpoint(
        history_id,
        next_page_token,
        Some("history"),
        Some(start_history_id),
    )
}

fn gmail_checkpoint(
    history_id: Option<String>,
    next_page_token: Option<String>,
    page_kind: Option<&'static str>,
    start_history_id: Option<&str>,
) -> Option<Value> {
    let history_id = history_id?;
    let mut checkpoint = json!({
        "provider": "gmail",
        "history_id": history_id
    });

    if let Some(next_page_token) = next_page_token {
        checkpoint["next_page_token"] = json!(next_page_token);
        if let Some(page_kind) = page_kind {
            checkpoint["page_kind"] = json!(page_kind);
        }
        if let Some(start_history_id) = start_history_id {
            checkpoint["start_history_id"] = json!(start_history_id);
        }
    }

    Some(checkpoint)
}

pub(super) fn imap_checkpoint(
    mailbox: &str,
    uid_validity: Option<u32>,
    latest_uid: Option<u32>,
) -> Value {
    let mut checkpoint = json!({
        "provider": "imap",
        "mailbox": mailbox
    });

    if let Some(uid_validity) = uid_validity {
        checkpoint["uid_validity"] = json!(uid_validity);
    }
    if let Some(latest_uid) = latest_uid {
        checkpoint["last_seen_uid"] = json!(latest_uid);
    }

    checkpoint
}

pub(super) fn next_imap_uid_floor(last_seen_uid: Option<u32>) -> Option<u32> {
    match last_seen_uid {
        Some(uid) => uid.checked_add(1),
        None => Some(1),
    }
}

pub(super) fn imap_uid_search_query(first_uid: u32) -> String {
    format!("UID {first_uid}:*")
}

pub(super) fn retain_uids_from_floor(uids: Vec<u32>, first_uid: u32) -> Vec<u32> {
    uids.into_iter().filter(|uid| *uid >= first_uid).collect()
}

pub(super) fn uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn select_uids_for_fetch(
    mut uids: Vec<u32>,
    max_messages: usize,
    latest_messages: bool,
) -> Vec<u32> {
    uids.sort_unstable();
    if latest_messages && uids.len() > max_messages {
        uids[uids.len() - max_messages..].to_vec()
    } else {
        uids.truncate(max_messages);
        uids
    }
}

pub(super) fn sha256_fingerprint<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }

    format!("sha256:{}", hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        gmail_history_checkpoint, imap_uid_search_query, next_imap_uid_floor,
        retain_uids_from_floor, select_uids_for_fetch,
    };

    #[test]
    fn select_uids_for_fetch_keeps_latest_window_when_requested() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, true),
            vec![42, 43]
        );
    }

    #[test]
    fn select_uids_for_fetch_keeps_oldest_window_for_sync_default() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, false),
            vec![41, 42]
        );
    }

    #[test]
    fn imap_uid_search_uses_uid_criterion_after_checkpoint() {
        let first_uid = next_imap_uid_floor(Some(30144)).expect("next UID");

        assert_eq!(first_uid, 30145);
        assert_eq!(imap_uid_search_query(first_uid), "UID 30145:*");
    }

    #[test]
    fn imap_uid_search_starts_at_first_uid_without_checkpoint() {
        let first_uid = next_imap_uid_floor(None).expect("first UID");

        assert_eq!(first_uid, 1);
        assert_eq!(imap_uid_search_query(first_uid), "UID 1:*");
    }

    #[test]
    fn imap_uid_search_discards_star_wraparound_uid() {
        assert_eq!(
            retain_uids_from_floor(vec![30144], 30145),
            Vec::<u32>::new()
        );
        assert_eq!(
            retain_uids_from_floor(vec![30144, 30145, 30146], 30145),
            vec![30145, 30146]
        );
    }

    #[test]
    fn imap_uid_floor_stops_at_u32_max() {
        assert_eq!(next_imap_uid_floor(Some(u32::MAX)), None);
    }

    #[test]
    fn gmail_history_checkpoint_preserves_pagination_origin() {
        assert_eq!(
            gmail_history_checkpoint(
                "history-start",
                Some("history-latest".to_owned()),
                Some("history-next".to_owned()),
            ),
            Some(json!({
                "provider": "gmail",
                "history_id": "history-latest",
                "next_page_token": "history-next",
                "page_kind": "history",
                "start_history_id": "history-start"
            }))
        );
    }
}
