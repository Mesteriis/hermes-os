use serde_json::Value;

use super::errors::YandexTelemostProtocolError;
use super::models::{YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR};

pub(crate) fn validate_required(
    field: &'static str,
    value: &str,
) -> Result<String, YandexTelemostProtocolError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(YandexTelemostProtocolError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value.to_owned())
}

pub(crate) fn validate_json_object(
    field: &'static str,
    value: &Value,
) -> Result<(), YandexTelemostProtocolError> {
    if !value.is_object() {
        return Err(YandexTelemostProtocolError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(crate) fn validate_api_base_url(
    value: Option<&str>,
) -> Result<String, YandexTelemostProtocolError> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(YANDEX_TELEMOST_API_BASE_URL);
    if !(value.starts_with("https://")
        || value.starts_with("http://127.0.0.1")
        || value.starts_with("http://localhost"))
    {
        return Err(YandexTelemostProtocolError::InvalidRequest(
            "Yandex Telemost API base URL must be HTTPS, localhost, or 127.0.0.1".to_owned(),
        ));
    }
    Ok(value.trim_end_matches('/').to_owned())
}

pub fn validate_telemost_join_url(value: &str) -> Result<String, YandexTelemostProtocolError> {
    let value = validate_required("join_url", value)?;
    if !value.starts_with("https://") {
        return Err(YandexTelemostProtocolError::InvalidRequest(
            "Yandex Telemost join URL must be HTTPS".to_owned(),
        ));
    }
    let host = value
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();
    match host {
        "telemost.yandex.ru" | "telemost.yandex.com" => Ok(value),
        _ => Err(YandexTelemostProtocolError::InvalidRequest(format!(
            "unsupported Yandex Telemost join URL host `{host}`"
        ))),
    }
}

pub fn yandex_telemost_oauth_token_secret_ref(account_id: &str) -> String {
    let stable = account_id
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("provider/{YANDEX_TELEMOST_PROVIDER_KIND_STR}/{stable}/oauth-token")
}

pub fn sanitize_yandex_telemost_payload(payload: Value) -> Value {
    match payload {
        Value::Object(mut object) => {
            for key in [
                "access_token",
                "authorization",
                "oauth_token",
                "token",
                "refresh_token",
                "cookie",
                "cookies",
                "password",
                "secret",
                "audio_bytes",
                "raw_audio",
                "mp3_bytes",
            ] {
                object.remove(key);
            }
            Value::Object(
                object
                    .into_iter()
                    .map(|(key, value)| (key, sanitize_yandex_telemost_payload(value)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_yandex_telemost_payload)
                .collect(),
        ),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn join_url_rejects_non_telemost_hosts() {
        let error = validate_telemost_join_url("https://evil.example/room").unwrap_err();
        assert!(error.to_string().contains("unsupported Yandex Telemost"));
    }

    #[test]
    fn sanitizer_removes_secret_and_audio_material_recursively() {
        let sanitized = sanitize_yandex_telemost_payload(json!({
            "id": "c1",
            "oauth_token": "secret",
            "nested": { "mp3_bytes": "base64", "speaker": "Alice" }
        }));
        assert_eq!(sanitized["id"], "c1");
        assert!(sanitized.get("oauth_token").is_none());
        assert!(sanitized["nested"].get("mp3_bytes").is_none());
    }
}
