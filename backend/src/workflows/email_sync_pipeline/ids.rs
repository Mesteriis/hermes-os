pub(super) const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

pub(super) fn raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    let mut encoded = String::from("raw:v1:");
    append_raw_record_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, record_kind);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_raw_record_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}
