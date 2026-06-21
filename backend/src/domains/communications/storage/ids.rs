pub(crate) fn mail_blob_id(sha256: &str) -> String {
    format!("blob:v1:{sha256}")
}

pub(crate) fn mail_attachment_id(message_id: &str, provider_attachment_id: &str) -> String {
    let mut encoded = String::from("att:v1:");
    append_id_component(&mut encoded, message_id);
    encoded.push(':');
    append_id_component(&mut encoded, provider_attachment_id);
    encoded
}

pub(crate) fn communication_attachment_import_id(seed: &str) -> String {
    let mut encoded = String::from("att-import:v1:");
    append_id_component(&mut encoded, seed);
    encoded
}

fn append_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}
