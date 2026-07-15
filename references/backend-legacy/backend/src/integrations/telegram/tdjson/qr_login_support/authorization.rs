use serde_json::Value;

pub(in crate::integrations::telegram::tdjson) fn password_hint(
    authorization_state: &Value,
) -> Option<String> {
    authorization_state
        .get("password_hint")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(in crate::integrations::telegram::tdjson) fn state_allows_qr_request(state_type: &str) -> bool {
    matches!(
        state_type,
        "authorizationStateWaitPhoneNumber"
            | "authorizationStateWaitPremiumPurchase"
            | "authorizationStateWaitEmailAddress"
            | "authorizationStateWaitEmailCode"
            | "authorizationStateWaitCode"
            | "authorizationStateWaitRegistration"
    )
}
