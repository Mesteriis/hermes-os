use super::{
    ProviderAccount, ProviderAccountSecretPurpose, Value, WhatsAppProviderRuntimeShape,
    WhatsAppRuntimeStatus,
};

pub(super) fn account_provider_shape(
    _account: &ProviderAccount,
    _fallback: WhatsAppProviderRuntimeShape,
) -> WhatsAppProviderRuntimeShape {
    WhatsAppProviderRuntimeShape::WebCompanion
}

pub(super) fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

pub(super) fn provider_shape_restorable_secret_purpose(
    _provider_shape: WhatsAppProviderRuntimeShape,
) -> ProviderAccountSecretPurpose {
    ProviderAccountSecretPurpose::WhatsappWebSessionKey
}

pub(super) fn runtime_blockers(runtime_kind: &str, last_error: Option<&str>) -> Vec<String> {
    let mut blockers = Vec::new();
    if runtime_kind != "fixture" {
        blockers.push("live_whatsapp_runtime_blocked".to_owned());
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push(error.trim().to_owned());
    }
    blockers
}

pub(super) fn qr_pair_code_blockers(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
) -> Vec<String> {
    let mut blockers = vec![
        "whatsapp_qr_pairing_requires_visible_runtime".to_owned(),
        "live_whatsapp_runtime_blocked".to_owned(),
    ];
    if runtime_kind == "fixture" {
        blockers.push("fixture_runtime_cannot_link_live_accounts".to_owned());
    }
    if runtime_kind != "fixture" && !provider_shape_runtime_feature_enabled(provider_shape) {
        blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
    }
    blockers
}

pub(super) fn provider_command_blockers(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    session_restore_available: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if runtime_kind == "fixture" {
        blockers.push("fixture_runtime_does_not_execute_provider_commands".to_owned());
    } else {
        blockers.push("live_whatsapp_runtime_blocked".to_owned());
        blockers.push("whatsapp_live_provider_execution_missing".to_owned());
        if !provider_shape_runtime_feature_enabled(provider_shape) {
            blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
        }
    }
    if !session_restore_available {
        blockers.push("whatsapp_session_restore_unavailable".to_owned());
    }
    blockers
}

pub(super) fn whatsapp_account_lifecycle_state(account: &ProviderAccount) -> &str {
    account
        .config
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("created")
}

pub(super) fn runtime_runtime_available(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    status: &str,
) -> bool {
    runtime_kind != "fixture"
        && runtime_kind != "live_blocked"
        && provider_shape_runtime_feature_enabled(provider_shape)
        && matches!(status, "linked" | "available" | "syncing" | "degraded")
}

pub(super) fn media_transfer_available(
    runtime_kind: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    status: &str,
) -> bool {
    runtime_runtime_available(runtime_kind, provider_shape, status)
        && matches!(status, "available" | "degraded")
}

pub(super) fn runtime_health_status(status: &WhatsAppRuntimeStatus) -> &'static str {
    if !status.runtime_blockers.is_empty() {
        return "blocked";
    }
    if status.status == "available" && status.live_runtime_available && status.live_send_available {
        return "available";
    }
    "degraded"
}

pub(super) fn runtime_status_blockers(
    status: &str,
    provider_shape: WhatsAppProviderRuntimeShape,
    runtime_kind: &str,
    session_restore_available: bool,
    last_error: Option<&str>,
) -> Vec<String> {
    let mut blockers = Vec::new();
    match status {
        "link_required" | "created" => blockers.push("whatsapp_session_link_required".to_owned()),
        "qr_pending" | "pair_code_pending" => {
            blockers.extend(qr_pair_code_blockers(runtime_kind, provider_shape));
        }
        "revoked" => {
            blockers.push("whatsapp_session_revoked".to_owned());
        }
        "removed" => {
            blockers.push("whatsapp_account_removed".to_owned());
        }
        "blocked" => {
            blockers.push("live_whatsapp_runtime_blocked".to_owned());
        }
        _ => {}
    }
    if status == "link_required" && runtime_kind != "fixture" {
        blockers.push("whatsapp_hidden_webview_runtime_required".to_owned());
    }
    if runtime_kind != "fixture" && !provider_shape_runtime_feature_enabled(provider_shape) {
        blockers.push(provider_shape_runtime_feature_blocker(provider_shape).to_owned());
    }
    if !session_restore_available
        && !matches!(
            status,
            "revoked" | "removed" | "blocked" | "available" | "linked"
        )
    {
        blockers.push("whatsapp_session_restore_unavailable".to_owned());
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push("whatsapp_runtime_error_present".to_owned());
    }
    blockers
}

pub(super) fn provider_shape_runtime_feature_enabled(
    provider_shape: WhatsAppProviderRuntimeShape,
) -> bool {
    provider_shape == WhatsAppProviderRuntimeShape::WebCompanion
}

pub(super) fn authorized_session_runtime_kind(account: &ProviderAccount, extra: &Value) -> String {
    extra
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| account_runtime_kind(account))
}

pub(super) fn provider_shape_runtime_feature_blocker(
    _provider_shape: WhatsAppProviderRuntimeShape,
) -> &'static str {
    "whatsapp_web_runtime_feature_unavailable"
}
