use hermes_provider_zoom::protocol::{
    sanitize_zoom_payload, validate_array, validate_non_empty, validate_object,
    zoom_authorization_url,
};
use serde_json::json;

#[test]
fn validation_preserves_field_context() {
    let error = validate_non_empty("account_id", " ").unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid Zoom request: account_id must not be empty"
    );
}

#[test]
fn json_shape_validation_rejects_the_wrong_shape() {
    assert!(validate_object("metadata", &json!([])).is_err());
    assert!(validate_array("segments", &json!({})).is_err());
}

#[test]
fn authorization_url_encodes_protocol_parameters() {
    let url = zoom_authorization_url(
        "https://zoom.example/oauth/authorize",
        "client-id",
        "http://localhost/callback",
        &["meeting:read".to_owned()],
        "state-token",
    )
    .expect("authorization URL");
    assert!(url.contains("response_type=code"));
    assert!(url.contains("scope=meeting%3Aread"));
}

#[test]
fn sanitizer_removes_secret_like_fields_recursively() {
    let sanitized = sanitize_zoom_payload(json!({
        "access_token": "secret",
        "nested": { "api_key": "secret", "safe": true }
    }));
    assert!(sanitized.get("access_token").is_none());
    assert!(sanitized["nested"].get("api_key").is_none());
    assert_eq!(sanitized["nested"]["safe"], true);
}
