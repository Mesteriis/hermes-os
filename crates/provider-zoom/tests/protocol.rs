use hermes_provider_zoom::protocol::{validate_array, validate_non_empty, validate_object};
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
