pub(crate) fn message_id(account_id: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("msg:v1:");
    append_message_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_message_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_message_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}
