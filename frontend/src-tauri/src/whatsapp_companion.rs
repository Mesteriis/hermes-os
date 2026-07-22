use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use hermes_whatsapp_api::host_bridge::{
    HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION,
    WhatsAppHostBridgeEnvelopeV1, WhatsAppHostObservationV1,
};
use hermes_whatsapp_api::WhatsAppProviderCommand;

const PROVIDER_SHAPE: &str = "whatsapp_web_companion";
const RUNTIME_KIND: &str = "webview_companion";
const WINDOW_LABEL_PREFIX: &str = "whatsapp-companion";
const WHATSAPP_WEB_URL: &str = "https://web.whatsapp.com/";
const RUNTIME_EVENTS_BRIDGE_PATH: &str =
    "whatsapp.client://observation/runtime-events";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRequest {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionCommandPollRequest {
    pub(crate) account_id: String,
    pub(crate) host_claim_id: String,
    pub(crate) limit: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionCommandFailureRequest {
    pub(crate) account_id: String,
    pub(crate) operation_id: String,
    pub(crate) host_claim_id: String,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionCommandResultRequest {
    pub(crate) account_id: String,
    pub(crate) operation_id: String,
    pub(crate) provider_event_id: String,
    pub(crate) provider_request_id: Option<String>,
    pub(crate) succeeded: bool,
    pub(crate) observed_at: String,
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
    pub(crate) reused_existing_window: bool,
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
    pub(crate) result_path: &'static str,
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
    pub(crate) runtime_bridge_status: String,
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
pub(crate) async fn start_hidden_whatsapp_webview(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let window_label = companion_window_label(&request.account_id)?;
    if let Some(_window) = app.get_webview_window(&window_label) {
        return manifest_for_account(&request.account_id, false, true);
    }

    let url = WHATSAPP_WEB_URL
        .parse()
        .map_err(|error| format!("invalid WhatsApp Web URL: {error}"))?;
    let initialization_script =
        companion_initialization_script(&request.account_id, &window_label)?;
    WebviewWindowBuilder::new(&app, window_label, WebviewUrl::External(url))
        .title("WhatsApp hidden WebView runtime")
        .visible(false)
        .resizable(false)
        .initialization_script(initialization_script)
        .on_navigation(|url| url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com"))
        .build()
        .map_err(|error| format!("failed to start hidden WhatsApp WebView: {error}"))?;

    manifest_for_account(&request.account_id, true, false)
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_relay_observation(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionRelayObservationRequest,
) -> Result<WhatsAppWebCompanionRelayObservationReceipt, String> {
    let account_id = required_account_id(&request.account_id)?.to_owned();
    let expected_window_label = companion_window_label(&account_id)?;
    if webview_window.label() != expected_window_label {
        return Err(format!(
            "WhatsApp companion relay rejected caller window {} for account {}",
            webview_window.label(),
            account_id
        ));
    }

    let event_family = required_ascii_slug("event_family", &request.event_family)?;
    let provider_event_id = required_ascii_slug("provider_event_id", &request.provider_event_id)?;
    let observed_at_unix_seconds = required_observed_at(&request.observed_at)?;
    let typed_runtime_bridge_path =
        runtime_bridge_path_for_event_family(event_family).ok_or_else(|| {
            format!("unsupported WhatsApp companion relay event family {event_family}")
        })?;
    let sanitized_metadata = sanitize_relay_metadata(request.metadata);
    let runtime_event_kind = format!("webview_companion.{event_family}.observed");
    let import_batch_id = runtime_bridge_import_batch_id(&account_id, provider_event_id);
    let envelope = WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: account_id.to_owned(),
        provider_event_id: provider_event_id.to_owned(),
        observed_at_unix_seconds,
        observation: host_observation_for_event_family(event_family, &sanitized_metadata)?,
    };
    let runtime_bridge_status = tauri::async_runtime::spawn_blocking(move || {
        crate::whatsapp_runtime_client::dispatch_host_observation(envelope)
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
        relay_state: "accepted_by_whatsapp_runtime",
        relay_channel: "tauri_versioned_whatsapp_host_bridge_unix_socket",
        sanitized_metadata,
        runtime_event_kind,
        import_batch_id,
        runtime_bridge_status,
        event_flow: "hidden_webview_companion -> tauri_versioned_host_bridge -> whatsapp_runtime -> provider_observation_projection",
        completion_rule: "provider_observed_event_reconciliation_required",
    })
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_poll_commands(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionCommandPollRequest,
) -> Result<Vec<WhatsAppProviderCommand>, String> {
    let account_id = required_account_id(&request.account_id)?.to_owned();
    let host_claim_id = required_ascii_slug("host_claim_id", &request.host_claim_id)?.to_owned();
    let expected_window_label = companion_window_label(&account_id)?;
    if webview_window.label() != expected_window_label {
        return Err(format!(
            "WhatsApp command poll rejected caller window {} for account {}",
            webview_window.label(), account_id
        ));
    }
    if request.limit == 0 || request.limit > 50 {
        return Err("WhatsApp command poll limit must be between 1 and 50".to_owned());
    }
    tauri::async_runtime::spawn_blocking(move || {
        crate::whatsapp_runtime_client::poll_pending_commands(account_id, host_claim_id, request.limit)
    })
    .await
    .map_err(|error| format!("WhatsApp command poll task failed: {error}"))?
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_report_command_failure(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionCommandFailureRequest,
) -> Result<(), String> {
    let account_id = required_account_id(&request.account_id)?.to_owned();
    let expected_window_label = companion_window_label(&account_id)?;
    if webview_window.label() != expected_window_label {
        return Err("WhatsApp command failure caller window is not admitted".to_owned());
    }
    let operation_id = required_ascii_slug("operation_id", &request.operation_id)?.to_owned();
    let host_claim_id = required_ascii_slug("host_claim_id", &request.host_claim_id)?.to_owned();
    let reason = request.reason.trim().to_owned();
    if reason.is_empty() || reason.len() > 512 {
        return Err("WhatsApp command failure reason is invalid".to_owned());
    }
    tauri::async_runtime::spawn_blocking(move || {
        crate::whatsapp_runtime_client::report_command_failure(operation_id, host_claim_id, reason)
    })
    .await
    .map_err(|error| format!("WhatsApp command failure task failed: {error}"))?
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_report_command_result(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionCommandResultRequest,
) -> Result<String, String> {
    let account_id = required_account_id(&request.account_id)?.to_owned();
    let expected_window_label = companion_window_label(&account_id)?;
    if webview_window.label() != expected_window_label {
        return Err("WhatsApp command result caller window is not admitted".to_owned());
    }
    let operation_id = required_ascii_slug("operation_id", &request.operation_id)?.to_owned();
    let provider_event_id = required_ascii_slug("provider_event_id", &request.provider_event_id)?.to_owned();
    let observed_at_unix_seconds = required_observed_at(&request.observed_at)?;
    if let Some(provider_request_id) = request.provider_request_id.as_deref() {
        required_ascii_slug("provider_request_id", provider_request_id)?;
    }
    let envelope = WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id,
        provider_event_id,
        observed_at_unix_seconds,
        observation: WhatsAppHostObservationV1::CommandResult {
            operation_id,
            provider_request_id: request.provider_request_id,
            succeeded: request.succeeded,
        },
    };
    tauri::async_runtime::spawn_blocking(move || {
        crate::whatsapp_runtime_client::dispatch_host_observation(envelope)
    })
    .await
    .map_err(|error| format!("WhatsApp command result dispatch task failed: {error}"))?
}

fn manifest_for_account(
    account_id: &str,
    opened_window: bool,
    reused_existing_window: bool,
) -> Result<WhatsAppWebCompanionManifest, String> {
    Ok(WhatsAppWebCompanionManifest {
        account_id: required_account_id(account_id)?.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        driver_id: "tauri_hidden_webview_companion",
        window_label: companion_window_label(account_id)?,
        target_url: WHATSAPP_WEB_URL,
        opened_window,
        reused_existing_window,
        owner_visible: false,
        hidden_headless_mode: "required_tauri_webview_not_headless_browser",
        tauri_ipc_available_to_companion_window: true,
        event_flow: "hidden_webview_companion -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation",
        event_extractor: WhatsAppWebCompanionExtractorContract {
            state: "contract_injected_relay_dispatch_available",
            initialization_script: "installed_on_hidden_companion_webview",
            script_scope: "main_frame_only",
            origin_guard: "https://web.whatsapp.com",
            navigation_guard: "https://web.whatsapp.com_only",
        relay_channel: "tauri_versioned_whatsapp_host_bridge_unix_socket",
        runtime_bridge_dispatch: "versioned_whatsapp_runtime_socket_wired",
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
            authorized_session_path: "whatsapp.client://observation/sessions/authorized",
            runtime_event_path: "whatsapp.client://observation/runtime-events",
            sync_lifecycle_path: "whatsapp.client://observation/sync-lifecycle",
            message_paths: vec![
                "whatsapp.client://observation/messages",
                "whatsapp.client://observation/message-updates",
                "whatsapp.client://observation/message-deletes",
                "whatsapp.client://observation/receipts",
                "whatsapp.client://observation/reactions",
                "whatsapp.client://observation/statuses",
                "whatsapp.client://observation/status-views",
                "whatsapp.client://observation/status-deletes",
            ],
            conversation_paths: vec![
                "whatsapp.client://observation/dialogs",
                "whatsapp.client://observation/participants",
                "whatsapp.client://observation/presence",
                "whatsapp.client://observation/calls",
            ],
            media_paths: vec![
                "whatsapp.client://observation/media",
                "whatsapp.client://observation/media-lifecycle",
            ],
        },
        command_channel: WhatsAppWebCompanionCommandChannel {
            kind: "durable_outbox",
            claim_path: "whatsapp.client://observation/commands/claim",
            failure_path: "whatsapp.client://observation/commands/{command_id}/failed",
            result_path: "whatsapp.client://observation/commands/{command_id}/result",
            completion_rule: "provider_observed_event_reconciliation_required",
        },
        secret_policy: WhatsAppWebCompanionSecretPolicy {
            session_material: "host_vault_only_via_authorized_session_bridge",
            cookies: "not_read_or_returned_by_tauri_command",
            browser_profile_secrets: "not_read_or_returned_by_tauri_command",
            qr_pair_code_artifacts: "never_returned_by_hidden_webview_runtime",
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
    relay_channel: "tauri_versioned_whatsapp_host_bridge_unix_socket",
    runtime_bridge_dispatch: "versioned_whatsapp_runtime_socket_wired",
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
        "runtime_lifecycle" => Some("whatsapp.client://observation/runtime-events"),
        "sync_lifecycle" => Some("whatsapp.client://observation/sync-lifecycle"),
        "message" => Some("whatsapp.client://observation/messages"),
        "message_update" => Some("whatsapp.client://observation/message-updates"),
        "message_delete" => Some("whatsapp.client://observation/message-deletes"),
        "receipt" => Some("whatsapp.client://observation/receipts"),
        "reaction" => Some("whatsapp.client://observation/reactions"),
        "dialog" => Some("whatsapp.client://observation/dialogs"),
        "participant" => Some("whatsapp.client://observation/participants"),
        "presence" => Some("whatsapp.client://observation/presence"),
        "call_metadata" => Some("whatsapp.client://observation/calls"),
        "status" => Some("whatsapp.client://observation/statuses"),
        "status_view" => Some("whatsapp.client://observation/status-views"),
        "status_delete" => Some("whatsapp.client://observation/status-deletes"),
        "media" => Some("whatsapp.client://observation/media"),
        "media_lifecycle" => Some("whatsapp.client://observation/media-lifecycle"),
        _ => None,
    }
}

fn host_observation_for_event_family(
    event_family: &str,
    metadata: &Value,
) -> Result<WhatsAppHostObservationV1, String> {
    let required = |key: &str| metadata_string(metadata, key);
    match event_family {
        "runtime_lifecycle" | "sync_lifecycle" | "media_lifecycle" => {
            Ok(WhatsAppHostObservationV1::RuntimeState { state: required("state")? })
        }
        "message" => Ok(WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
            sender_id: required("sender_id")?,
        }),
        "message_update" => Ok(WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
        }),
        "message_delete" => Ok(WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
        }),
        "receipt" => Ok(WhatsAppHostObservationV1::Receipt {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
            delivery_state: required("delivery_state")?,
        }),
        "reaction" => Ok(WhatsAppHostObservationV1::Reaction {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
            actor_id: required("actor_id")?,
            emoji: metadata.get("emoji").and_then(Value::as_str).map(str::to_owned),
            is_active: metadata
                .get("is_active")
                .and_then(Value::as_bool)
                .ok_or_else(|| "is_active metadata is required for WhatsApp reaction".to_owned())?,
        }),
        "dialog" => Ok(WhatsAppHostObservationV1::Dialog {
            provider_chat_id: required("provider_chat_id")?,
            title: required("title")?,
            kind: required("kind")?,
        }),
        "participant" => Ok(WhatsAppHostObservationV1::Participant {
            provider_chat_id: required("provider_chat_id")?,
            provider_identity_id: required("provider_identity_id")?,
            display_name: required("display_name")?,
        }),
        "presence" => Ok(WhatsAppHostObservationV1::Presence {
            provider_chat_id: required("provider_chat_id")?,
            provider_identity_id: required("provider_identity_id")?,
            state: required("state")?,
        }),
        "call_metadata" => Ok(WhatsAppHostObservationV1::CallMetadata {
            provider_call_id: required("provider_call_id")?,
            provider_chat_id: required("provider_chat_id")?,
            direction: required("direction")?,
            state: required("state")?,
        }),
        "status" => Ok(WhatsAppHostObservationV1::StatusMetadata {
            provider_status_id: required("provider_status_id")?,
            sender_id: required("sender_id")?,
        }),
        "status_view" => Ok(WhatsAppHostObservationV1::StatusViewMetadata {
            provider_status_id: required("provider_status_id")?,
            viewer_id: required("viewer_id")?,
        }),
        "status_delete" => Ok(WhatsAppHostObservationV1::StatusDeletedMetadata {
            provider_status_id: required("provider_status_id")?,
        }),
        "media" => Ok(WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id: required("provider_chat_id")?,
            provider_message_id: required("provider_message_id")?,
            provider_media_id: required("provider_media_id")?,
            media_kind: required("media_kind")?,
        }),
        _ => Err(format!("unsupported WhatsApp host observation family: {event_family}")),
    }
}

fn metadata_string(metadata: &Value, key: &str) -> Result<String, String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{key} metadata is required for WhatsApp host observation"))
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

fn required_observed_at(observed_at: &str) -> Result<i64, String> {
    let observed_at = observed_at.trim();
    if observed_at.is_empty() {
        return Err("observed_at is required for WhatsApp companion relay".to_owned());
    }
    observed_at.parse::<i64>().map_err(|_| {
        "observed_at must be Unix seconds for WhatsApp host bridge v1".to_owned()
    })
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
        assert!(!manifest.owner_visible);
        assert_eq!(manifest.tauri_ipc_available_to_companion_window, true);
        assert_eq!(
            manifest.event_extractor.state,
            "contract_injected_relay_dispatch_available"
        );
        assert_eq!(
            manifest.event_extractor.relay_channel,
            "tauri_versioned_whatsapp_host_bridge_unix_socket"
        );
        assert_eq!(
            manifest.event_extractor.runtime_bridge_dispatch,
            "versioned_whatsapp_runtime_socket_wired"
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
            "whatsapp.client://observation/sessions/authorized"
        );
        assert!(
            manifest
                .bridge_routes
                .message_paths
                .iter()
                .all(|path| { path.starts_with("whatsapp.client://observation/") })
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
        assert!(script.contains("tauri_versioned_whatsapp_host_bridge_unix_socket"));
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
            Some("whatsapp.client://observation/messages")
        );
        assert_eq!(
            runtime_bridge_path_for_event_family("media_lifecycle"),
            Some("whatsapp.client://observation/media-lifecycle")
        );
        assert_eq!(runtime_bridge_path_for_event_family("unknown"), None);
    }

    #[test]
    fn relay_host_observation_is_typed_and_metadata_only() {
        let sanitized_metadata = sanitize_relay_metadata(serde_json::json!({
            "provider_chat_id": "chat-1",
            "provider_message_id": "message-1",
            "sender_id": "sender-1",
            "message_body": "private"
        }));
        let observation = host_observation_for_event_family("message", &sanitized_metadata)
            .expect("typed observation");
        assert_eq!(
            observation,
            WhatsAppHostObservationV1::MessageIdentity {
                provider_chat_id: "chat-1".to_owned(),
                provider_message_id: "message-1".to_owned(),
                sender_id: "sender-1".to_owned(),
            }
        );
        assert!(!serde_json::to_string(&observation).unwrap().contains("private"));
    }
}
