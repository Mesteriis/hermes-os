use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::models::ParsedRfc822Message;
use super::text::non_empty_recipients;

pub(super) fn redact_message(
    provider_record_id: &str,
    source_fingerprint: &str,
    occurred_at: Option<DateTime<Utc>>,
    parsed: ParsedRfc822Message,
) -> ParsedRfc822Message {
    let mut recipients = BTreeSet::new();
    for recipient in parsed.to {
        recipients.insert(redacted_email("recipient", &recipient));
    }

    ParsedRfc822Message {
        subject: format!("Redacted subject {}", short_hash(&parsed.subject)),
        from: redacted_email("sender", &parsed.from),
        to: non_empty_recipients(recipients.into_iter().collect()),
        body_text: redacted_body_text(
            provider_record_id,
            source_fingerprint,
            occurred_at,
            &parsed.body_text,
        ),
    }
}

fn redacted_email(prefix: &str, input: &str) -> String {
    format!("{prefix}-{}@example.invalid", short_hash(input))
}

fn redacted_body_text(
    provider_record_id: &str,
    source_fingerprint: &str,
    occurred_at: Option<DateTime<Utc>>,
    body_text: &str,
) -> String {
    let occurred_at = occurred_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_owned());
    format!(
        "Redacted body fixture for provider_record_id_hash={}, source_fingerprint={}, occurred_at={}, original_chars={}.",
        short_hash(provider_record_id),
        source_fingerprint,
        occurred_at,
        body_text.chars().count()
    )
}

fn short_hash(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    hex_lower(&digest[..6])
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
