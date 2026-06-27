use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "whatsapp_web_companion";
const RUNTIME_KIND: &str = "webview_companion";
const WINDOW_LABEL_PREFIX: &str = "whatsapp-companion";
const WHATSAPP_WEB_URL: &str = "https://web.whatsapp.com/";
const DEFAULT_BACKEND_HTTP_ADDR: &str = "127.0.0.1:8080";
const RUNTIME_EVENTS_BRIDGE_PATH: &str =
    "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRequest {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) driver_id: &'static str,
    pub(crate) window_label: String,
    pub(crate) target_url: &'static str,
    pub(crate) opened_window: bool,
    pub(crate) focused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) tauri_ipc_available_to_companion_window: bool,
    pub(crate) event_flow: &'static str,
    pub(crate) event_extractor: WhatsAppWebCompanionExtractorContract,
    pub(crate) bridge_routes: WhatsAppWebCompanionBridgeRoutes,
    pub(crate) command_channel: WhatsAppWebCompanionCommandChannel,
    pub(crate) secret_policy: WhatsAppWebCompanionSecretPolicy,
    pub(crate) remaining_blockers: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionExtractorContract {
    pub(crate) state: &'static str,
    pub(crate) initialization_script: &'static str,
    pub(crate) script_scope: &'static str,
    pub(crate) origin_guard: &'static str,
    pub(crate) navigation_guard: &'static str,
    pub(crate) relay_channel: &'static str,
    pub(crate) runtime_bridge_dispatch: &'static str,
    pub(crate) allowed_observations: Vec<&'static str>,
    pub(crate) forbidden_reads: Vec<&'static str>,
    pub(crate) next_gate: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionBridgeRoutes {
    pub(crate) authorized_session_path: &'static str,
    pub(crate) runtime_event_path: &'static str,
    pub(crate) sync_lifecycle_path: &'static str,
    pub(crate) message_paths: Vec<&'static str>,
    pub(crate) conversation_paths: Vec<&'static str>,
    pub(crate) media_paths: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionCommandChannel {
    pub(crate) kind: &'static str,
    pub(crate) claim_path: &'static str,
    pub(crate) failure_path: &'static str,
    pub(crate) completion_rule: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionSecretPolicy {
    pub(crate) session_material: &'static str,
    pub(crate) cookies: &'static str,
    pub(crate) browser_profile_secrets: &'static str,
    pub(crate) qr_pair_code_artifacts: &'static str,
    pub(crate) message_bodies: &'static str,
    pub(crate) media_bytes: &'static str,
    pub(crate) postgres_storage: &'static str,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRelayObservationRequest {
    pub(crate) account_id: String,
    pub(crate) event_family: String,
    pub(crate) provider_event_id: String,
    pub(crate) observed_at: String,
    #[serde(default)]
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionRelayObservationReceipt {
    pub(crate) account_id: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) window_label: String,
    pub(crate) event_family: String,
    pub(crate) provider_event_id: String,
    pub(crate) observed_at: String,
    pub(crate) target_runtime_bridge_path: &'static str,
    pub(crate) typed_runtime_bridge_path: &'static str,
    pub(crate) relay_state: &'static str,
    pub(crate) relay_channel: &'static str,
    pub(crate) sanitized_metadata: Value,
    pub(crate) runtime_event_kind: String,
    pub(crate) import_batch_id: String,
    pub(crate) runtime_bridge_http_status: u16,
    pub(crate) event_flow: &'static str,
    pub(crate) completion_rule: &'static str,
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_manifest(
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    manifest_for_account(&request.account_id, false, false)
}

#[tauri::command]
pub(crate) async fn open_whatsapp_web_companion(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let window_label = companion_window_label(&request.account_id)?;
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|error| format!("failed to show WhatsApp companion window: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("failed to focus WhatsApp companion window: {error}"))?;
        return manifest_for_account(&request.account_id, false, true);
    }

    let url = WHATSAPP_WEB_URL
        .parse()
        .map_err(|error| format!("invalid WhatsApp Web URL: {error}"))?;
    let initialization_script =
        companion_initialization_script(&request.account_id, &window_label)?;
    let window = WebviewWindowBuilder::new(&app, window_label, WebviewUrl::External(url))
        .title("WhatsApp Web Companion")
        .visible(true)
        .resizable(true)
        .inner_size(1160.0, 780.0)
        .initialization_script(initialization_script)
        .on_navigation(|url| url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com"))
        .build()
        .map_err(|error| format!("failed to open WhatsApp companion window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus WhatsApp companion window: {error}"))?;

    manifest_for_account(&request.account_id, true, false)
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_relay_observation(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionRelayObservationRequest,
) -> Result<WhatsAppWebCompanionRelayObservationReceipt, String> {
    let account_id = required_account_id(&request.account_id)?;
    let expected_window_label = companion_window_label(account_id)?;
    if webview_window.label() != expected_window_label {
        return Err(format!(
            "WhatsApp companion relay rejected caller window {} for account {}",
            webview_window.label(),
            account_id
        ));
    }

    let event_family = required_ascii_slug("event_family", &request.event_family)?;
    let provider_event_id = required_ascii_slug("provider_event_id", &request.provider_event_id)?;
    let observed_at = required_observed_at(&request.observed_at)?;
    let typed_runtime_bridge_path =
        runtime_bridge_path_for_event_family(event_family).ok_or_else(|| {
            format!("unsupported WhatsApp companion relay event family {event_family}")
        })?;
    let sanitized_metadata = sanitize_relay_metadata(request.metadata);
    let runtime_event_kind = format!("webview_companion.{event_family}.observed");
    let import_batch_id = runtime_bridge_import_batch_id(account_id, provider_event_id);
    let runtime_event_payload = runtime_bridge_runtime_event_payload(
        account_id,
        event_family,
        provider_event_id,
        observed_at,
        typed_runtime_bridge_path,
        &runtime_event_kind,
        &import_batch_id,
        sanitized_metadata.clone(),
    );
    let runtime_bridge_http_status = tauri::async_runtime::spawn_blocking(move || {
        dispatch_runtime_bridge_runtime_event(runtime_event_payload)
    })
    .await
    .map_err(|error| format!("WhatsApp companion relay dispatch task failed: {error}"))??;

    Ok(WhatsAppWebCompanionRelayObservationReceipt {
        account_id: account_id.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        window_label: expected_window_label,
        event_family: event_family.to_owned(),
        provider_event_id: provider_event_id.to_owned(),
        observed_at: observed_at.to_owned(),
        target_runtime_bridge_path: RUNTIME_EVENTS_BRIDGE_PATH,
        typed_runtime_bridge_path,
        relay_state: "dispatched_to_backend_runtime_bridge_runtime_event",
        relay_channel: "tauri_allowlisted_companion_runtime_bridge_dispatch",
        sanitized_metadata,
        runtime_event_kind,
        import_batch_id,
        runtime_bridge_http_status,
        event_flow: "visible_webview_companion -> tauri_allowlisted_relay_preflight -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation",
        completion_rule: "provider_observed_event_reconciliation_required",
    })
}

fn manifest_for_account(
    account_id: &str,
    opened_window: bool,
    focused_existing_window: bool,
) -> Result<WhatsAppWebCompanionManifest, String> {
    Ok(WhatsAppWebCompanionManifest {
        account_id: required_account_id(account_id)?.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        driver_id: "tauri_visible_webview_companion",
        window_label: companion_window_label(account_id)?,
        target_url: WHATSAPP_WEB_URL,
        opened_window,
        focused_existing_window,
        owner_visible: true,
        hidden_headless_mode: "forbidden",
        tauri_ipc_available_to_companion_window: true,
        event_flow: "visible_webview_companion -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation",
        event_extractor: WhatsAppWebCompanionExtractorContract {
            state: "contract_injected_relay_dispatch_available",
            initialization_script: "installed_on_visible_companion_window",
            script_scope: "main_frame_only",
            origin_guard: "https://web.whatsapp.com",
            navigation_guard: "https://web.whatsapp.com_only",
            relay_channel: "tauri_allowlisted_companion_runtime_bridge_dispatch",
            runtime_bridge_dispatch: "runtime_events_bridge_wired_smoke_pending",
            allowed_observations: vec![
                "runtime_lifecycle_metadata",
                "sync_lifecycle_metadata",
                "message_identity_metadata",
                "receipt_metadata",
                "reaction_metadata",
                "dialog_metadata",
                "participant_metadata",
                "presence_metadata",
                "call_metadata",
                "status_metadata",
                "media_metadata_without_bytes",
            ],
            forbidden_reads: vec![
                "cookies",
                "web_storage",
                "indexed_db",
                "browser_profile_secrets",
                "session_material",
                "message_bodies",
                "media_bytes",
            ],
            next_gate: "manual_live_smoke_before_public_availability",
        },
        bridge_routes: WhatsAppWebCompanionBridgeRoutes {
            authorized_session_path: "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            runtime_event_path: "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            sync_lifecycle_path: "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            message_paths: vec![
                "/api/v1/integrations/whatsapp/runtime-bridge/messages",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
                "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
                "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
                "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
            ],
            conversation_paths: vec![
                "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
                "/api/v1/integrations/whatsapp/runtime-bridge/participants",
                "/api/v1/integrations/whatsapp/runtime-bridge/presence",
                "/api/v1/integrations/whatsapp/runtime-bridge/calls",
            ],
            media_paths: vec![
                "/api/v1/integrations/whatsapp/runtime-bridge/media",
                "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
            ],
        },
        command_channel: WhatsAppWebCompanionCommandChannel {
            kind: "durable_outbox",
            claim_path: "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            failure_path: "/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed",
            completion_rule: "provider_observed_event_reconciliation_required",
        },
        secret_policy: WhatsAppWebCompanionSecretPolicy {
            session_material: "host_vault_only_via_authorized_session_bridge",
            cookies: "not_read_or_returned_by_tauri_command",
            browser_profile_secrets: "not_read_or_returned_by_tauri_command",
            qr_pair_code_artifacts: "owner_visible_runtime_only",
            message_bodies: "excluded_from_manifest_and_health",
            media_bytes: "local_blob_storage_only_not_manifest_or_postgres",
            postgres_storage: "metadata_bindings_only_no_session_cookie_or_profile_secret",
        },
        remaining_blockers: vec![
            "whatsapp_webview_runtime_panel_action_not_implemented",
            "whatsapp_webview_live_smoke_required",
        ],
    })
}

fn companion_initialization_script(account_id: &str, window_label: &str) -> Result<String, String> {
    let account_id_json = serde_json::to_string(required_account_id(account_id)?)
        .map_err(|error| format!("failed to serialize WhatsApp account id: {error}"))?;
    let window_label_json = serde_json::to_string(window_label)
        .map_err(|error| format!("failed to serialize WhatsApp window label: {error}"))?;

    Ok(format!(
        r#"
(() => {{
  const allowedOrigin = "https://web.whatsapp.com";
  if (window.location.origin !== allowedOrigin) {{
    return;
  }}

  const contract = Object.freeze({{
    version: 1,
    provider_shape: "{PROVIDER_SHAPE}",
    runtime_kind: "{RUNTIME_KIND}",
    account_id: {account_id_json},
    window_label: {window_label_json},
    relay_command: "whatsapp_web_companion_relay_observation",
    relay_channel: "tauri_allowlisted_companion_runtime_bridge_dispatch",
    runtime_bridge_dispatch: "runtime_events_bridge_wired_smoke_pending",
    payload_policy: "metadata_only_no_private_content_or_secret_material"
  }});

  Object.defineProperty(window, "__HERMES_WHATSAPP_COMPANION__", {{
    value: contract,
    configurable: false,
    enumerable: false,
    writable: false
  }});

  window.dispatchEvent(new CustomEvent("hermes-whatsapp-companion-contract-ready", {{
    detail: Object.freeze({{
      version: contract.version,
      provider_shape: contract.provider_shape,
      runtime_kind: contract.runtime_kind,
      relay_channel: contract.relay_channel
    }})
  }}));
}})();
"#
    ))
}

fn runtime_bridge_path_for_event_family(event_family: &str) -> Option<&'static str> {
    match event_family {
        "runtime_lifecycle" => Some("/api/v1/integrations/whatsapp/runtime-bridge/runtime-events"),
        "sync_lifecycle" => Some("/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle"),
        "message" => Some("/api/v1/integrations/whatsapp/runtime-bridge/messages"),
        "message_update" => Some("/api/v1/integrations/whatsapp/runtime-bridge/message-updates"),
        "message_delete" => Some("/api/v1/integrations/whatsapp/runtime-bridge/message-deletes"),
        "receipt" => Some("/api/v1/integrations/whatsapp/runtime-bridge/receipts"),
        "reaction" => Some("/api/v1/integrations/whatsapp/runtime-bridge/reactions"),
        "dialog" => Some("/api/v1/integrations/whatsapp/runtime-bridge/dialogs"),
        "participant" => Some("/api/v1/integrations/whatsapp/runtime-bridge/participants"),
        "presence" => Some("/api/v1/integrations/whatsapp/runtime-bridge/presence"),
        "call_metadata" => Some("/api/v1/integrations/whatsapp/runtime-bridge/calls"),
        "status" => Some("/api/v1/integrations/whatsapp/runtime-bridge/statuses"),
        "status_view" => Some("/api/v1/integrations/whatsapp/runtime-bridge/status-views"),
        "status_delete" => Some("/api/v1/integrations/whatsapp/runtime-bridge/status-deletes"),
        "media" => Some("/api/v1/integrations/whatsapp/runtime-bridge/media"),
        "media_lifecycle" => Some("/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle"),
        _ => None,
    }
}

fn runtime_bridge_runtime_event_payload(
    account_id: &str,
    event_family: &str,
    provider_event_id: &str,
    observed_at: &str,
    typed_runtime_bridge_path: &str,
    runtime_event_kind: &str,
    import_batch_id: &str,
    sanitized_metadata: Value,
) -> Value {
    json!({
        "account_id": account_id,
        "provider_event_id": provider_event_id,
        "runtime_event_kind": runtime_event_kind,
        "runtime_status": "observed",
        "lifecycle_state": "provider_observed_metadata",
        "severity": "info",
        "metadata": {
            "source": "webview_companion_allowlisted_relay",
            "provider_shape": PROVIDER_SHAPE,
            "runtime_kind": RUNTIME_KIND,
            "event_family": event_family,
            "typed_runtime_bridge_path": typed_runtime_bridge_path,
            "payload_policy": "sanitized_metadata_only_no_message_bodies_or_media_bytes",
            "projection_policy": "runtime_event_evidence_only_until_richer_typed_payload",
            "sanitized_metadata": sanitized_metadata,
        },
        "import_batch_id": import_batch_id,
        "observed_at": observed_at,
    })
}

fn dispatch_runtime_bridge_runtime_event(payload: Value) -> Result<u16, String> {
    let url = local_backend_runtime_bridge_url(RUNTIME_EVENTS_BRIDGE_PATH)?;
    let secret = local_api_secret()?;
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    match agent
        .post(&url)
        .header("X-Hermes-Secret", &secret)
        .send_json(&payload)
    {
        Ok(response) => Ok(response.status().as_u16()),
        Err(ureq::Error::StatusCode(status)) => Err(format!(
            "WhatsApp companion relay backend dispatch rejected runtime event with HTTP status {status}"
        )),
        Err(error) => Err(format!(
            "WhatsApp companion relay backend dispatch failed: {error}"
        )),
    }
}

fn local_backend_runtime_bridge_url(path: &str) -> Result<String, String> {
    if !path.starts_with("/api/v1/integrations/whatsapp/runtime-bridge/") {
        return Err("WhatsApp companion relay target must be a runtime-bridge path".to_owned());
    }
    let raw_addr = std::env::var("HERMES_HTTP_ADDR")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_BACKEND_HTTP_ADDR.to_owned());
    let base_url = if raw_addr.starts_with("http://") {
        raw_addr
    } else {
        format!("http://{raw_addr}")
    };
    if !is_allowed_local_backend_url(&base_url) {
        return Err(
            "WhatsApp companion relay backend dispatch requires a loopback HTTP address".to_owned(),
        );
    }
    Ok(format!("{}{}", base_url.trim_end_matches('/'), path))
}

fn is_allowed_local_backend_url(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://localhost:")
        || url.starts_with("http://[::1]:")
}

fn local_api_secret() -> Result<String, String> {
    let secret = std::env::var("HERMES_LOCAL_API_SECRET")
        .unwrap_or_else(|_| "change-me-local-api-secret".to_owned());
    let secret = secret.trim();
    if secret.is_empty() {
        return Err(
            "HERMES_LOCAL_API_SECRET is required for WhatsApp companion relay dispatch".to_owned(),
        );
    }
    Ok(secret.to_owned())
}

fn runtime_bridge_import_batch_id(account_id: &str, provider_event_id: &str) -> String {
    format!("whatsapp-webview-companion:{account_id}:{provider_event_id}")
}

fn required_ascii_slug<'a>(field_name: &str, value: &'a str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!(
            "{field_name} is required for WhatsApp companion relay"
        ));
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
    {
        return Err(format!(
            "{field_name} contains unsupported characters for WhatsApp companion relay"
        ));
    }
    Ok(value)
}

fn required_observed_at(observed_at: &str) -> Result<&str, String> {
    let observed_at = observed_at.trim();
    if observed_at.is_empty() {
        return Err("observed_at is required for WhatsApp companion relay".to_owned());
    }
    Ok(observed_at)
}

fn sanitize_relay_metadata(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter_map(|(key, value)| {
                    if is_forbidden_relay_metadata_key(&key) {
                        None
                    } else {
                        Some((key, sanitize_relay_metadata(value)))
                    }
                })
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_relay_metadata)
                .collect::<Vec<Value>>(),
        ),
        primitive => primitive,
    }
}

fn is_forbidden_relay_metadata_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "access_token",
        "api_token",
        "app_secret",
        "auth",
        "body",
        "browser_profile",
        "caption",
        "content",
        "cookie",
        "credential",
        "direct_path",
        "indexed_db",
        "local_storage",
        "media_bytes",
        "media_key",
        "message_body",
        "password",
        "raw",
        "secret",
        "session",
        "session_storage",
        "static_url",
        "text",
        "token",
        "url",
        "web_storage",
    ]
    .iter()
    .any(|forbidden| key.contains(forbidden))
}

fn companion_window_label(account_id: &str) -> Result<String, String> {
    let account_id = required_account_id(account_id)?;
    let mut sanitized = String::with_capacity(account_id.len());
    let mut previous_dash = false;
    for character in account_id.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_dash = false;
            character.to_ascii_lowercase()
        } else if character == '-' || character == '_' {
            if previous_dash {
                continue;
            }
            previous_dash = character == '-';
            character
        } else if !previous_dash {
            previous_dash = true;
            '-'
        } else {
            continue;
        };
        sanitized.push(next);
        if sanitized.len() >= 64 {
            break;
        }
    }
    let sanitized = sanitized.trim_matches('-').trim_matches('_');
    if sanitized.is_empty() {
        return Err("account_id does not contain a valid window label segment".to_owned());
    }
    Ok(format!("{WINDOW_LABEL_PREFIX}-{sanitized}"))
}

fn required_account_id(account_id: &str) -> Result<&str, String> {
    let account_id = account_id.trim();
    if account_id.is_empty() {
        return Err("account_id is required for WhatsApp companion runtime".to_owned());
    }
    Ok(account_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn companion_window_label_is_account_scoped_and_stable() {
        assert_eq!(
            companion_window_label(" Wa Live/Primary ").expect("label"),
            "whatsapp-companion-wa-live-primary"
        );
        assert_eq!(
            companion_window_label("wa_live_primary").expect("label"),
            "whatsapp-companion-wa_live_primary"
        );
    }

    #[test]
    fn manifest_exposes_only_event_spine_and_secret_policy() {
        let manifest = manifest_for_account("wa-1", true, false).expect("manifest");

        assert_eq!(manifest.provider_shape, "whatsapp_web_companion");
        assert_eq!(manifest.runtime_kind, "webview_companion");
        assert_eq!(manifest.owner_visible, true);
        assert_eq!(manifest.tauri_ipc_available_to_companion_window, true);
        assert_eq!(
            manifest.event_extractor.state,
            "contract_injected_relay_dispatch_available"
        );
        assert_eq!(
            manifest.event_extractor.relay_channel,
            "tauri_allowlisted_companion_runtime_bridge_dispatch"
        );
        assert_eq!(
            manifest.event_extractor.runtime_bridge_dispatch,
            "runtime_events_bridge_wired_smoke_pending"
        );
        assert!(
            manifest
                .event_extractor
                .forbidden_reads
                .contains(&"message_bodies")
        );
        assert_eq!(
            manifest.command_channel.completion_rule,
            "provider_observed_event_reconciliation_required"
        );
        assert_eq!(
            manifest.bridge_routes.authorized_session_path,
            "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized"
        );
        assert!(
            manifest
                .bridge_routes
                .message_paths
                .iter()
                .all(|path| { path.starts_with("/api/v1/integrations/whatsapp/runtime-bridge/") })
        );
        assert_eq!(
            manifest.secret_policy.cookies,
            "not_read_or_returned_by_tauri_command"
        );
        assert!(
            manifest
                .remaining_blockers
                .contains(&"whatsapp_webview_runtime_panel_action_not_implemented")
        );
    }

    #[test]
    fn companion_initialization_script_is_origin_guarded_and_metadata_only() {
        let script =
            companion_initialization_script("wa-1", "whatsapp-companion-wa-1").expect("script");

        assert!(script.contains("window.location.origin !== allowedOrigin"));
        assert!(script.contains("__HERMES_WHATSAPP_COMPANION__"));
        assert!(script.contains("whatsapp_web_companion_relay_observation"));
        assert!(script.contains("tauri_allowlisted_companion_runtime_bridge_dispatch"));
        assert!(script.contains("metadata_only_no_private_content_or_secret_material"));
        for forbidden in [
            "document.cookie",
            "localStorage",
            "sessionStorage",
            "indexedDB",
            "fetch(",
            "XMLHttpRequest",
            "postMessage",
            "__TAURI__",
            "invoke(",
        ] {
            assert!(
                !script.contains(forbidden),
                "companion init script must not use forbidden API {forbidden}"
            );
        }
    }

    #[test]
    fn relay_metadata_sanitizer_removes_secret_like_and_private_content_keys() {
        let sanitized = sanitize_relay_metadata(serde_json::json!({
            "provider_chat_id": "chat-1",
            "message_body": "private",
            "nested": {
                "receipt_state": "read",
                "media_key": "secret-media-key",
                "direct_path": "/secret"
            },
            "items": [
                {"provider_message_id": "m1", "text": "private text"},
                {"reaction": "thumbs_up"}
            ]
        }));

        assert_eq!(sanitized["provider_chat_id"], serde_json::json!("chat-1"));
        assert_eq!(
            sanitized["nested"]["receipt_state"],
            serde_json::json!("read")
        );
        assert_eq!(
            sanitized["items"][0]["provider_message_id"],
            serde_json::json!("m1")
        );
        assert_eq!(
            sanitized["items"][1]["reaction"],
            serde_json::json!("thumbs_up")
        );
        assert!(sanitized.get("message_body").is_none());
        assert!(sanitized["nested"].get("media_key").is_none());
        assert!(sanitized["nested"].get("direct_path").is_none());
        assert!(sanitized["items"][0].get("text").is_none());
    }

    #[test]
    fn relay_event_family_contract_maps_only_supported_runtime_bridge_paths() {
        assert_eq!(
            runtime_bridge_path_for_event_family("message"),
            Some("/api/v1/integrations/whatsapp/runtime-bridge/messages")
        );
        assert_eq!(
            runtime_bridge_path_for_event_family("media_lifecycle"),
            Some("/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle")
        );
        assert_eq!(runtime_bridge_path_for_event_family("unknown"), None);
    }

    #[test]
    fn relay_runtime_event_payload_is_metadata_only_and_targets_runtime_events() {
        let sanitized_metadata = sanitize_relay_metadata(serde_json::json!({
            "provider_chat_id": "chat-1",
            "message_body": "private",
            "receipt_state": "read"
        }));
        let payload = runtime_bridge_runtime_event_payload(
            "wa-1",
            "message",
            "provider-event-1",
            "2026-06-26T20:00:00Z",
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            "webview_companion.message.observed",
            "whatsapp-webview-companion:wa-1:provider-event-1",
            sanitized_metadata,
        );

        assert_eq!(
            RUNTIME_EVENTS_BRIDGE_PATH,
            payload_target_runtime_event_path()
        );
        assert_eq!(
            payload["runtime_event_kind"],
            serde_json::json!("webview_companion.message.observed")
        );
        assert_eq!(
            payload["metadata"]["typed_runtime_bridge_path"],
            serde_json::json!("/api/v1/integrations/whatsapp/runtime-bridge/messages")
        );
        assert_eq!(
            payload["metadata"]["sanitized_metadata"]["provider_chat_id"],
            serde_json::json!("chat-1")
        );
        assert_eq!(
            payload["metadata"]["sanitized_metadata"]["receipt_state"],
            serde_json::json!("read")
        );
        assert!(
            payload["metadata"]["sanitized_metadata"]
                .get("message_body")
                .is_none()
        );
        assert!(!payload.to_string().contains("private"));
    }

    #[test]
    fn relay_backend_dispatch_is_loopback_only() {
        assert!(is_allowed_local_backend_url("http://127.0.0.1:8080"));
        assert!(is_allowed_local_backend_url("http://localhost:8080"));
        assert!(is_allowed_local_backend_url("http://[::1]:8080"));
        assert!(!is_allowed_local_backend_url("https://127.0.0.1:8080"));
        assert!(!is_allowed_local_backend_url("http://192.168.1.10:8080"));
        assert!(!is_allowed_local_backend_url("https://web.whatsapp.com"));
    }
}

#[cfg(test)]
fn payload_target_runtime_event_path() -> &'static str {
    RUNTIME_EVENTS_BRIDGE_PATH
}
