use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

pub(super) fn raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
    source_fingerprint: &str,
    payload: &Value,
) -> String {
    let mut encoded = String::from("raw:v2:");
    append_raw_record_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, record_kind);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, provider_record_id);
    encoded.push(':');
    append_raw_record_id_component(
        &mut encoded,
        &snapshot_fingerprint(source_fingerprint, payload),
    );
    encoded
}

fn snapshot_fingerprint(source_fingerprint: &str, payload: &Value) -> String {
    let payload = serde_json::to_vec(payload).expect("mail provider payload is serializable");
    let mut hasher = Sha256::new();
    hasher.update(source_fingerprint.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    format!("sha256:{:x}", hasher.finalize())
}

fn append_raw_record_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::raw_record_id;

    #[test]
    fn raw_record_id_changes_when_provider_snapshot_metadata_changes() {
        let with_label = raw_record_id(
            "account",
            "email_message",
            "provider-message",
            "sha256:body",
            &json!({"label_ids": ["Inbox"]}),
        );
        let without_label = raw_record_id(
            "account",
            "email_message",
            "provider-message",
            "sha256:body",
            &json!({"label_ids": []}),
        );

        assert_ne!(with_label, without_label);
    }
}
