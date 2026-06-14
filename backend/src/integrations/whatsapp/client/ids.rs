use sha2::{Digest, Sha256};

pub(crate) fn whatsapp_web_session_id(account_id: &str) -> String {
    format!(
        "whatsapp_web_session:v5:{}",
        stable_hash(account_id.as_bytes())
    )
}

pub(crate) fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v5:whatsapp_web:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

pub(crate) fn whatsapp_web_raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    format!(
        "raw:v5:whatsapp_web:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
