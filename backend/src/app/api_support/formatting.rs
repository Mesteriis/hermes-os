use super::*;

pub(crate) use crate::platform::formatting::text_preview;

pub(crate) fn default_schema_version() -> i32 {
    1
}

pub(crate) fn empty_json_object() -> Value {
    json!({})
}

pub(crate) fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
