use super::*;

pub(super) fn runtime_status_from_account(account: ZoomAccount) -> ZoomRuntimeStatus {
    let checked_at = Utc::now();
    let mut blockers = account
        .config
        .get("runtime_blockers")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let last_error = account
        .config
        .get("last_error")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let fixture = account.auth_shape == "fixture";
    let live_authorized = !fixture && zoom_account_is_authorized(&account);
    let status = if account.lifecycle_state == "fixture_ready" {
        "stopped".to_owned()
    } else {
        account.lifecycle_state.clone()
    };
    let token_rotation = zoom_token_rotation_status(&account, checked_at);
    blockers.retain(|value| value != "zoom_provider_workers_not_enabled");
    if live_authorized {
        blockers.retain(|value| value != "zoom_live_authorization_required");
    }
    if live_authorized
        && token_rotation.rotation_required
        && !blockers
            .iter()
            .any(|value| value == ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER)
    {
        blockers.push(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER.to_owned());
    }
    let mut metadata = account.config.clone();
    metadata["token_rotation_policy"] = token_rotation.metadata;
    let healthy = account.lifecycle_state != "removed"
        && ((fixture && account.lifecycle_state != "removed")
            || (live_authorized && status == "running" && blockers.is_empty()));

    ZoomRuntimeStatus {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        runtime_kind: account.runtime_kind,
        status,
        healthy,
        auth_shape: account.auth_shape,
        live_runtime_available: live_authorized,
        recording_ingest_available: account.lifecycle_state != "removed",
        transcript_ingest_available: account.lifecycle_state != "removed",
        runtime_blockers: blockers,
        last_error,
        checked_at,
        metadata,
    }
}

pub(super) fn parse_optional_datetime(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|value| value.with_timezone(&Utc))
}

pub(super) struct ZoomTokenRotationStatus {
    rotation_required: bool,
    metadata: Value,
}

pub(super) fn zoom_token_rotation_status(
    account: &ZoomAccount,
    checked_at: DateTime<Utc>,
) -> ZoomTokenRotationStatus {
    let authorization = account.config.get("authorization");
    let expires_at = authorization
        .and_then(|value| value.get("expires_at"))
        .and_then(zoom_datetime_from_json);
    let token_secret_bound = authorization
        .and_then(|value| value.get("token_secret_bound"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let last_refresh_status = authorization
        .and_then(|value| value.get("last_token_refresh"))
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str);
    let last_refresh_failed = last_refresh_status == Some("failed");
    let expired = expires_at.is_some_and(|value| value <= checked_at);
    let refresh_due = expires_at.is_some_and(|value| {
        value
            <= checked_at
                + chrono::TimeDelta::seconds(ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS)
    });
    let live_authorized = account.auth_shape != ZoomAuthShape::Fixture.as_str()
        && zoom_account_is_authorized(account);
    let missing_token_secret = live_authorized && !token_secret_bound;
    let rotation_required = last_refresh_failed || expired || missing_token_secret;
    let status = if rotation_required {
        "required"
    } else if refresh_due {
        "due"
    } else if live_authorized {
        "current"
    } else {
        "not_applicable"
    };

    ZoomTokenRotationStatus {
        rotation_required,
        metadata: json!({
            "status": status,
            "rotation_required": rotation_required,
            "refresh_due": refresh_due,
            "expired": expired,
            "missing_token_secret": missing_token_secret,
            "last_refresh_status": last_refresh_status,
            "expires_at": expires_at,
            "checked_at": checked_at,
            "policy": {
                "explicit_refresh_threshold_seconds": ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS,
                "maintenance_refresh_threshold_seconds": ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS,
                "max_refresh_threshold_seconds": ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS,
                "provider_expiry_safety_margin_seconds": ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS,
                "failure_blocker": ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER,
            },
        }),
    }
}

fn zoom_datetime_from_json(value: &Value) -> Option<DateTime<Utc>> {
    value
        .as_str()
        .and_then(|candidate| DateTime::parse_from_rfc3339(candidate).ok())
        .map(|value| value.with_timezone(&Utc))
}

pub(super) fn add_zoom_token_refresh_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let blocker = Value::String(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER.to_owned());
    if !blockers.iter().any(|value| value == &blocker) {
        blockers.push(blocker);
    }
    config["runtime_blockers"] = json!(blockers);
}

pub(super) fn clear_zoom_token_refresh_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER));
    config["runtime_blockers"] = json!(blockers);
}

pub(super) fn clear_zoom_provider_workers_not_enabled_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some("zoom_provider_workers_not_enabled"));
    config["runtime_blockers"] = json!(blockers);
}

pub(super) fn clear_zoom_live_authorization_required_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some("zoom_live_authorization_required"));
    config["runtime_blockers"] = json!(blockers);
}

pub(super) fn zoom_account_is_authorized(account: &ZoomAccount) -> bool {
    account
        .config
        .get("authorization")
        .and_then(|authorization| authorization.get("status"))
        .and_then(Value::as_str)
        == Some("authorized")
}

pub(super) fn ensure_zoom_account_is_authorized(account: &ZoomAccount) -> Result<(), ZoomError> {
    if zoom_account_is_authorized(account) {
        return Ok(());
    }
    Err(ZoomError::InvalidRequest(format!(
        "Zoom account `{}` is not authorized",
        account.account_id
    )))
}
