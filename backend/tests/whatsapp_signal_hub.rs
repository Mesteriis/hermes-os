use std::fs;
use std::path::{Path, PathBuf};

use hermes_hub_backend::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};
use serde_json::json;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend has repository parent")
        .to_path_buf()
}

#[test]
fn whatsapp_signal_hub_fixture_matrix_covers_event_families() {
    let root = repo_root();
    let signal_hub_whatsapp = read(root.join("backend/src/domains/signal_hub/whatsapp.rs"));
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler.clone() + &read_all_sources(root.join("backend/src/application"));
    let fixture_matrix = read(root.join("docs/whatsapp/fixture-test-matrix.md"));

    for fixture in signal_hub_fixtures() {
        if let Some(raw_record_kind) = fixture.raw_record_kind {
            assert!(
                signal_hub_whatsapp.contains(&format!(
                    "\"{raw_record_kind}\" => \"{}\"",
                    fixture.signal_kind
                )),
                "Signal Hub raw WhatsApp mapping missing for {raw_record_kind}"
            );
        } else {
            assert!(
                signal_hub_whatsapp.contains("_ => \"message\""),
                "Signal Hub raw WhatsApp mapping must default unknown message-like records to message"
            );
        }

        assert!(
            signal_hub_whatsapp.contains("signal.raw.whatsapp.{event_kind}.observed"),
            "WhatsApp raw signals must use the Signal Hub raw observed event family"
        );
        assert!(
            fixture_matrix.contains(fixture.matrix_label),
            "WhatsApp fixture-test matrix must document {}",
            fixture.matrix_label
        );
        assert!(
            event_producers.contains(fixture.realtime_event_constant),
            "WhatsApp event producers must emit realtime event constant {}",
            fixture.realtime_event_constant
        );
    }
}

#[test]
fn whatsapp_runtime_lifecycle_event_fixtures_are_sanitized_and_complete() {
    let root = repo_root();
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler + &read_all_sources(root.join("backend/src/application"));
    let event_bus = read(root.join("backend/src/platform/events/bus.rs"));

    for (required, handler_marker) in [
        (
            whatsapp_event_types::RUNTIME_STATUS_CHANGED,
            "RUNTIME_STATUS_CHANGED",
        ),
        (whatsapp_event_types::RUNTIME_EVENT, "RUNTIME_EVENT"),
        (
            whatsapp_event_types::SESSION_LINK_STATE_CHANGED,
            "SESSION_LINK_STATE_CHANGED",
        ),
        (whatsapp_event_types::SYNC_STARTED, "SYNC_STARTED"),
        (whatsapp_event_types::SYNC_PROGRESS, "SYNC_PROGRESS"),
        (whatsapp_event_types::SYNC_COMPLETED, "SYNC_COMPLETED"),
        (whatsapp_event_types::SYNC_FAILED, "SYNC_FAILED"),
        (
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            "COMMAND_STATUS_CHANGED",
        ),
        (
            whatsapp_event_types::COMMAND_RECONCILED,
            "COMMAND_RECONCILED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_REQUESTED,
            "MEDIA_UPLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_STARTED,
            "whatsapp.media.upload.started",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_PROGRESS,
            "whatsapp.media.upload.progress",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_COMPLETED,
            "whatsapp.media.upload.completed",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_FAILED,
            "MEDIA_UPLOAD_FAILED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED,
            "MEDIA_DOWNLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_STARTED,
            "whatsapp.media.download.started",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_PROGRESS,
            "whatsapp.media.download.progress",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_COMPLETED,
            "whatsapp.media.download.completed",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_FAILED,
            "MEDIA_DOWNLOAD_FAILED",
        ),
    ] {
        assert!(
            event_bus.contains(required),
            "WhatsApp event bus constants must expose {required}"
        );
        assert!(
            event_producers.contains(handler_marker),
            "WhatsApp provider handler/runtime bridge or worker must emit {required}"
        );
    }

    let sanitized = sanitize_event_payload(json!({
        "account_id": "wa-1",
        "session_key": "secret-session",
        "access_token": "secret-token",
        "password": "secret-password",
        "raw_body": "private body",
        "safe": "kept"
    }));

    assert_eq!(sanitized["safe"], json!("kept"));
    for secret_key in ["session_key", "access_token", "password", "raw_body"] {
        assert!(
            sanitized.get(secret_key).is_none(),
            "sanitized WhatsApp event payload must remove {secret_key}"
        );
    }
}

#[test]
fn whatsapp_live_smoke_evidence_requires_typed_sanitized_refs() {
    let root = repo_root();
    let evidence_validator = read(root.join("scripts/whatsapp-live-smoke-evidence.mjs"));
    let readiness = read(root.join("scripts/whatsapp-live-smoke-readiness.mjs"));
    let checklist = read(root.join("docs/whatsapp/live-smoke-checklist.md"));
    let status = read(root.join("docs/whatsapp/status.md"));

    assert!(
        evidence_validator.contains("allowedEvidenceRefPrefixes")
            && evidence_validator.contains("requiredEvidenceRefPrefixGroups")
            && evidence_validator.contains("evidence_refs")
            && evidence_validator.contains("'raw_record:'")
            && evidence_validator.contains("'event_log:'")
            && evidence_validator.contains("'signal_hub:'")
            && evidence_validator.contains("'command:'")
            && evidence_validator.contains("'vault_binding:'")
            && evidence_validator.contains("'blob:'")
            && evidence_validator.contains("'edge_proxy:'")
            && evidence_validator.contains("'log_scan:'"),
        "Live-smoke evidence must require typed sanitized references for raw evidence, events, commands, vault bindings, media, edge proxy and redaction checks"
    );
    assert!(
        evidence_validator
            .contains("'commands.no_completion_without_provider_observed_evidence': [")
            && evidence_validator.contains("['event_log:', 'signal_hub:']")
            && evidence_validator.contains("gateId.startsWith('outbound.')")
            && evidence_validator.contains("return [['command:'], ['event_log:', 'signal_hub:']]")
            && evidence_validator.contains("weak_reconciliation_refs_fail")
            && evidence_validator.contains("placeholder_refs_fail")
            && evidence_validator
                .contains("evidence.${gateId}.evidence_refs must include at least one"),
        "Live-smoke evidence must reject weak provider-write evidence that lacks command plus provider-observed event refs"
    );
    assert!(
        checklist.contains("Each passed gate must also include concrete sanitized `evidence_refs`")
            && checklist.contains("`command:` plus observed event refs")
            && checklist.contains("`vault_binding:` for session/credential binding")
            && checklist.contains("Placeholder refs")
            && readiness.contains("strict live-smoke evidence references")
            && status.contains("65. `strict live-smoke evidence references`"),
        "Docs and readiness must expose the stricter live-smoke evidence contract used by the closure audit"
    );
}

#[test]
fn whatsapp_live_smoke_evidence_collector_is_not_a_bypass() {
    let root = repo_root();
    let collector = read(root.join("scripts/whatsapp-live-smoke-collect-evidence.mjs"));
    let evidence_validator = read(root.join("scripts/whatsapp-live-smoke-evidence.mjs"));
    let makefile = read(root.join("Makefile"));
    let readiness = read(root.join("scripts/whatsapp-live-smoke-readiness.mjs"));
    let checklist = read(root.join("docs/whatsapp/live-smoke-checklist.md"));
    let status = read(root.join("docs/whatsapp/status.md"));

    assert!(
        evidence_validator.contains("--provider-shape")
            && evidence_validator.contains("templateEvidence(providerShape, status)")
            && evidence_validator.contains("template status must be pending or passed"),
        "Evidence validator templates must be provider-shape aware so each runtime shape can produce its own smoke artifact"
    );
    assert!(
        collector
            .contains("defaultObservationsPath = '.local/whatsapp/live-smoke-observations.json'")
            && collector.contains("whatsapp-live-smoke-evidence.mjs")
            && collector.contains("--observations-template")
            && collector.contains("HERMES_WHATSAPP_SMOKE_ACCOUNT_ID")
            && collector.contains("sha256Fingerprint")
            && collector.contains("assertNoSecretLikeContent")
            && collector.contains("Gates without operator-provided sanitized refs remain pending")
            && collector.contains("mergeEvidence(template, observations)")
            && collector.contains("validateEvidence(filePath)")
            && collector.contains("process.exitCode = 1"),
        "Live-smoke collector must normalize sanitized observations and then fail through the strict validator until all gates are genuinely evidenced"
    );
    assert!(
        makefile.contains("whatsapp-live-smoke-collect-evidence:")
            && readiness.contains("manual_smoke_evidence_collector_contract")
            && checklist.contains("make whatsapp-live-smoke-collect-evidence")
            && checklist.contains("normalizer, not a bypass")
            && checklist.contains("Gates without operator-provided sanitized")
            && status.contains("67. `live-smoke evidence collector`")
            && status.contains("synthetic passed")
            && status.contains("gates."),
        "Makefile, readiness and docs must expose the collector as a non-bypass path for producing live-smoke artifacts"
    );
}

#[test]
fn whatsapp_native_md_upgrade_path_is_executable_evidence_not_assumption() {
    let root = repo_root();
    let gap_readiness = read(root.join("scripts/whatsapp-native-md-sdk-gap-readiness.mjs"));
    let cargo_toml = read(root.join("backend/Cargo.toml"));
    let cargo_lock = read(root.join("Cargo.lock"));
    let status = read(root.join("docs/whatsapp/status.md"));

    assert!(
        gap_readiness.contains("verifyRustAndCrateUpgradeContext()")
            && gap_readiness.contains("native_md_rust_baseline_context")
            && gap_readiness.contains("native_md_wa_rs_dependency_context")
            && gap_readiness.contains("native_md_crates_io_probe")
            && gap_readiness.contains("native_md_upgrade_requires_provider_api_not_toolchain_only")
            && gap_readiness.contains("HERMES_WA_RS_CRATES_IO_PROBE=1")
            && gap_readiness.contains("cargo info"),
        "Native MD gap readiness must make the Rust/wa-rs upgrade path executable evidence, not an assumption"
    );
    assert!(
        cargo_toml.contains("rust-version = \"1.89\"")
            && cargo_toml.contains("wa-rs = { version = \"0.2.0\"")
            && cargo_toml.contains("wa-rs-core = { version = \"0.2.0\"")
            && cargo_lock.contains("name = \"wa-rs\"\nversion = \"0.2.0\"")
            && cargo_lock.contains("name = \"wa-rs-core\"\nversion = \"0.2.0\""),
        "Native MD dependency evidence must stay pinned and visible while unsupported commands remain blocked"
    );
    assert!(
        status.contains("66. `native Rust/wa-rs upgrade path verifier`")
            && status.contains("Rust/toolchain upgrade is not treated as sufficient evidence")
            && status.contains("HERMES_WA_RS_CRATES_IO_PROBE=1")
            && status.contains("publish_status")
            && status.contains("join-by-invite"),
        "WhatsApp status must document that Rust/toolchain upgrade alone does not close the remaining native command gaps"
    );
}

#[test]
fn whatsapp_web_companion_bridge_contract_is_visible_event_spine_only() {
    let root = repo_root();
    let web_companion_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/web_companion.rs"));
    let runtime_source = read(root.join("backend/src/integrations/whatsapp/runtime/mod.rs"));
    let tauri_build_source = read(root.join("frontend/src-tauri/build.rs"));
    let tauri_cargo_toml = read(root.join("frontend/src-tauri/Cargo.toml"));
    let tauri_lib_source = read(root.join("frontend/src-tauri/src/lib.rs"));
    let tauri_companion_source = read(root.join("frontend/src-tauri/src/whatsapp_companion.rs"));
    let tauri_default_capability = read(root.join("frontend/src-tauri/capabilities/default.json"));
    let tauri_companion_capability =
        read(root.join("frontend/src-tauri/capabilities/whatsapp-companion-relay.json"));
    let frontend_companion_source =
        read(root.join("frontend/src/integrations/whatsapp/api/whatsappCompanion.ts"));
    let frontend_companion_test =
        read(root.join("frontend/src/integrations/whatsapp/api/whatsappCompanion.test.ts"));

    assert!(
        web_companion_source.contains("web_companion_bridge_contract_health_check")
            && runtime_source.contains("web_companion_bridge_contract_health_check")
            && runtime_source.contains("checks[\"web_companion_bridge\"]")
            && runtime_source.contains("checks[\"runtime\"][\"web_companion_bridge\"]"),
        "WhatsApp Web companion bridge contract must be surfaced through runtime health"
    );
    assert!(
        web_companion_source.contains("\"visible_runtime_required\": true")
            && web_companion_source.contains("\"hidden_headless_mode\": \"forbidden\"")
            && web_companion_source.contains("tauri_visible_webview_companion")
            && web_companion_source.contains("open_whatsapp_web_companion")
            && web_companion_source.contains("whatsapp_web_companion_manifest")
            && web_companion_source.contains("whatsapp_web_companion_relay_observation")
            && web_companion_source
                .contains("\"event_extractor\": \"contract_injected_relay_dispatch_available\"")
            && web_companion_source.contains(
                "\"relay_channel\": \"tauri_allowlisted_companion_runtime_bridge_dispatch\""
            )
            && web_companion_source.contains(
                "\"runtime_bridge_dispatch\": \"runtime_events_bridge_wired_smoke_pending\""
            )
            && web_companion_source
                .contains("\"dispatch_payload\": \"NewWhatsappWebRuntimeEvent\"")
            && web_companion_source.contains("\"domain_mutation\": \"forbidden\"")
            && web_companion_source.contains("\"command_completion\": \"forbidden\"")
            && web_companion_source.contains("whatsapp_visible_runtime_required")
            && web_companion_source.contains("manual_live_smoke_required"),
        "WhatsApp Web companion runtime must remain owner-visible and smoke-gated"
    );
    assert!(
        web_companion_source.contains("\"kind\": \"protected_runtime_bridge_routes\"")
            && web_companion_source
                .contains("/api/v1/integrations/whatsapp/runtime-bridge/messages")
            && web_companion_source
                .contains("/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle")
            && web_companion_source
                .contains("/api/v1/integrations/whatsapp/runtime-bridge/commands/claim")
            && web_companion_source.contains("provider_observed_event_reconciliation_required")
            && web_companion_source.contains("\"runtime_to_domain_calls\": \"forbidden\""),
        "WhatsApp Web companion bridge must use runtime-bridge events and durable outbox, not direct domain calls"
    );
    assert!(
        web_companion_source.contains("\"session_material\": \"excluded\"")
            && web_companion_source.contains("\"cookies\": \"excluded\"")
            && web_companion_source.contains("\"browser_profile_secrets\": \"excluded\"")
            && web_companion_source.contains("\"media_bytes\": \"excluded\"")
            && web_companion_source
                .contains("\"postgres_policy\": \"metadata_bindings_only_no_session_cookie_or_local_profile_secret\""),
        "WhatsApp Web companion bridge health contract must keep session/cookie/profile/media secrets out of health/events"
    );
    assert!(
        tauri_lib_source.contains("generate_handler!")
            && tauri_lib_source.contains("open_whatsapp_web_companion")
            && tauri_lib_source.contains("whatsapp_web_companion_manifest")
            && tauri_lib_source.contains("whatsapp_web_companion_relay_observation")
            && tauri_companion_source.contains("WebviewWindowBuilder::new")
            && tauri_companion_source.contains("WebviewUrl::External")
            && tauri_companion_source.contains("https://web.whatsapp.com/")
            && tauri_companion_source.contains(".initialization_script(initialization_script)")
            && tauri_companion_source.contains(".on_navigation(|url|")
            && tauri_companion_source.contains("url.host_str() == Some(\"web.whatsapp.com\")")
            && tauri_companion_source.contains("window.location.origin !== allowedOrigin")
            && tauri_companion_source.contains("__HERMES_WHATSAPP_COMPANION__")
            && tauri_companion_source.contains("tauri_ipc_available_to_companion_window: true")
            && tauri_companion_source
                .contains("tauri_allowlisted_companion_runtime_bridge_dispatch")
            && tauri_companion_source.contains("webview_window.label() != expected_window_label")
            && tauri_companion_source.contains("sanitize_relay_metadata")
            && tauri_companion_source.contains("runtime_bridge_runtime_event_payload")
            && tauri_companion_source.contains("dispatch_runtime_bridge_runtime_event")
            && tauri_companion_source.contains("RUNTIME_EVENTS_BRIDGE_PATH")
            && tauri_companion_source.contains("X-Hermes-Secret")
            && tauri_companion_source.contains("is_allowed_local_backend_url")
            && tauri_companion_source
                .contains("runtime_event_evidence_only_until_richer_typed_payload")
            && tauri_cargo_toml.contains("ureq")
            && tauri_companion_source.contains("not_read_or_returned_by_tauri_command")
            && tauri_companion_source
                .contains("/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized")
            && tauri_companion_source.contains("provider_observed_event_reconciliation_required")
            && tauri_companion_source
                .contains("whatsapp_webview_runtime_panel_action_not_implemented")
            && tauri_companion_source
                .contains("companion_initialization_script_is_origin_guarded_and_metadata_only")
            && tauri_companion_source.contains("!script.contains(forbidden)"),
        "WhatsApp Web companion must expose a visible Tauri producer shell that is limited to the runtime-bridge/event-spine contract"
    );
    assert!(
        tauri_default_capability.contains("\"main\"")
            && tauri_build_source
                .contains("tauri_build::AppManifest::new().commands(APP_COMMANDS)")
            && tauri_build_source.contains("\"open_whatsapp_web_companion\"")
            && tauri_build_source.contains("\"whatsapp_web_companion_manifest\"")
            && tauri_build_source.contains("\"whatsapp_web_companion_relay_observation\"")
            && tauri_default_capability.contains("allow-open-whatsapp-web-companion")
            && tauri_default_capability.contains("allow-whatsapp-web-companion-manifest")
            && !tauri_default_capability.contains("allow-whatsapp-web-companion-relay-observation")
            && tauri_companion_capability.contains("\"local\": false")
            && tauri_companion_capability.contains("https://web.whatsapp.com")
            && tauri_companion_capability.contains("whatsapp-companion-*")
            && tauri_companion_capability
                .contains("allow-whatsapp-web-companion-relay-observation")
            && !tauri_companion_capability.contains("core:default"),
        "WhatsApp companion Tauri ACL must allow only the relay dispatch command to the remote companion window"
    );
    assert!(
        frontend_companion_source.contains("from '@tauri-apps/api/core'")
            && frontend_companion_source.contains("open_whatsapp_web_companion")
            && frontend_companion_source.contains("whatsapp_web_companion_manifest")
            && frontend_companion_source.contains("whatsapp_web_companion_relay_observation")
            && frontend_companion_source.contains("WhatsAppWebCompanionManifest")
            && !frontend_companion_source.contains("ApiClient")
            && !frontend_companion_source.contains("fetch(")
            && frontend_companion_test.contains("expect(fetchMock).not.toHaveBeenCalled()")
            && frontend_companion_test.contains("contract_injected_relay_dispatch_available")
            && frontend_companion_test
                .contains("tauri_allowlisted_companion_runtime_bridge_dispatch")
            && frontend_companion_test
                .contains("dispatched_to_backend_runtime_bridge_runtime_event")
            && frontend_companion_test.contains("provider_observed_event_reconciliation_required")
            && frontend_companion_test.contains("not_read_or_returned_by_tauri_command"),
        "WhatsApp Web companion frontend bridge must use Tauri invoke directly and must not route through backend/domain HTTP APIs"
    );
}

#[test]
fn whatsapp_native_md_unsupported_write_gap_is_explicit_and_structured() {
    let root = repo_root();
    let native_md_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/native_md.rs"));
    let command_executor_source =
        read(root.join("backend/src/application/whatsapp_command_executor.rs"));
    let adr_0101 = read(root.join("docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md"));
    let whatsapp_gap_analysis = read(root.join("docs/whatsapp/gap-analysis.md"));

    assert!(
        native_md_source.contains("native_md_wa_rs_sdk_command_gap_health")
            && native_md_source
                .contains("\"evidence_basis\": \"local_crate_source_public_api_inventory\"")
            && native_md_source.contains("\"driver_version\": \"0.2.0\"")
            && native_md_source.contains("\"wa_rs_api\": \"Client::send_message\"")
            && native_md_source.contains("\"command_kind\": \"forward\"")
            && native_md_source.contains("\"submission_mode\": \"forwarded_text_reemit\"")
            && native_md_source.contains("native_md_forward_text_message")
            && native_md_source.contains("forwarding_score: Some(1)")
            && native_md_source.contains("is_forwarded: Some(true)")
            && native_md_source.contains("\"wa_rs_api\": \"Client::edit_message\"")
            && native_md_source.contains("\"wa_rs_api\": \"Client::revoke_message\"")
            && native_md_source.contains("\"wa_rs_api\": \"Client::mark_as_read\"")
            && native_md_source.contains("\"wa_rs_api\": \"Client::groups().leave\"")
            && native_md_source
                .contains("\"wa_rs_api\": \"Client::upload + Client::send_message\"")
            && native_md_source.contains("\"wa_rs_api\": \"Client::download_from_params\""),
        "Native MD health must expose the verified wa-rs SDK APIs behind the smoke-gated command subset"
    );
    assert!(
        !native_md_source.contains("no public forward_or_copy_message API found")
            && native_md_source.contains("\"command_kind\": \"publish_status\"")
            && native_md_source.contains("no public status_publish API found")
            && native_md_source.contains("\"command_kind\": \"archive\"")
            && native_md_source.contains("ArchiveUpdate is inbound app-state dispatch only")
            && native_md_source.contains("\"command_kind\": \"mute\"")
            && native_md_source.contains("MuteUpdate is inbound app-state dispatch only")
            && native_md_source.contains("\"command_kind\": \"pin\"")
            && native_md_source.contains("PinUpdate is inbound app-state dispatch only")
            && native_md_source.contains("\"command_kind\": \"mark_unread\"")
            && native_md_source.contains("MarkChatAsReadUpdate is inbound app-state dispatch only")
            && native_md_source.contains("\"command_kind\": \"join_group\"")
            && native_md_source.contains(
                "groups API exposes create/add/remove/admin/link/leave but no join-by-invite API"
            ),
        "Native MD unsupported writes must name the concrete wa-rs API gaps instead of implying live support"
    );
    assert!(
        adr_0101.contains("Forward is supported only as smoke-gated forwarded-text")
            && adr_0101.contains("reemit: Communications projection text")
            && adr_0101.contains("ContextInfo.is_forwarded = true")
            && adr_0101.contains("forwarding_score = 1")
            && adr_0101.contains("forwarded-text reemit contract for `forward`")
            && adr_0101
                .contains("archive/mute/pin/join/unread remain structured unsupported paths")
            && !adr_0101.contains("missing safe write APIs for forward")
            && !adr_0101.contains("Forward/status/archive/mute/pin/join/unread remain")
            && whatsapp_gap_analysis.contains("| Forwards | PARTIAL |")
            && whatsapp_gap_analysis.contains("forwarded-text reemit")
            && !whatsapp_gap_analysis.contains("| Forwards | MISSING |"),
        "ADR-0101 and gap analysis must not regress to treating forward as a fully unsupported native write after text reemit support"
    );
    assert!(
        native_md_source.contains("NATIVE_MD_UNSUPPORTED_COMMAND_ERROR_CODE")
            && native_md_source
                .contains("\"error_code\": NATIVE_MD_UNSUPPORTED_COMMAND_ERROR_CODE")
            && native_md_source.contains("fn native_md_unsupported_command_error")
            && native_md_source.contains("failed_before_runtime_driver_lookup")
            && native_md_source.contains(
                "claim_due_native_md_commands_for_execution_may_claim_to_write_structured_failure"
            )
            && native_md_source.contains("\"event_phase\": \"failed_before_provider_observation\"")
            && native_md_source.contains("\"retry_path\": \"terminal_dead_letter_without_retry\"")
            && native_md_source.contains("never_completed_without_provider_observed_event")
            && command_executor_source.contains("record_live_native_md_command_failure")
            && command_executor_source
                .contains("\"terminal_unsupported_before_provider_observation\"")
            && command_executor_source.contains("dead_letter_failed_command")
            && command_executor_source.contains("\"retry_policy\": retry_policy")
            && command_executor_source.contains("\"terminal\"")
            && command_executor_source.contains("error.error_code.as_deref()"),
        "Native MD unsupported writes must terminally dead-letter without retry and never complete from SDK failure"
    );

    let execute_start = native_md_source
        .find("pub(super) async fn execute_live_provider_command")
        .expect("native_md execute_live_provider_command");
    let execute_end = native_md_source[execute_start..]
        .find("pub(super) async fn decorate_runtime_health")
        .map(|offset| execute_start + offset)
        .expect("native_md execute_live_provider_command end");
    let execute_body = &native_md_source[execute_start..execute_end];
    let unsupported_preflight = execute_body
        .find("native_md_unsupported_command_error(&command.command_kind)")
        .expect("native_md unsupported command preflight");
    let smoke_gate = execute_body
        .find("native_md_live_smoke_opted_in")
        .expect("native_md smoke gate");
    let native_driver_call = execute_body
        .find("self.execute_native_command(command).await")
        .expect("native_md driver command call");
    assert!(
        unsupported_preflight < smoke_gate && smoke_gate < native_driver_call,
        "Native MD unsupported command kinds must fail with a deterministic unsupported error before smoke/runtime driver lookup can mask the command gap"
    );
}

#[test]
fn whatsapp_signal_hub_fixtures_do_not_complete_commands_from_sdk_success() {
    let root = repo_root();
    let runtime_source = read_all_sources(root.join("backend/src/integrations/whatsapp/runtime"));
    let command_executor_source =
        read(root.join("backend/src/application/whatsapp_command_executor.rs"));
    let reconciliation_source =
        read(root.join("backend/src/application/whatsapp_provider_observation_reconciliation.rs"));

    assert!(
        runtime_source.contains("reconciliation_status")
            && runtime_source.contains("provider_observed_at"),
        "WhatsApp commands must carry provider-observed reconciliation fields"
    );
    assert!(
        reconciliation_source.contains("signal.accepted.whatsapp")
            || reconciliation_source.contains("event_type"),
        "WhatsApp command reconciliation must consume accepted Signal Hub events"
    );
    assert!(
        runtime_source.contains("async fn mark_provider_command_reconciled")
            && runtime_source.contains("provider_observed_at = $2")
            && runtime_source.contains("reconciliation_status = 'observed'")
            && runtime_source.contains("completed_at = $2"),
        "WhatsApp runtime may complete commands only inside provider-observed reconciliation"
    );
    assert!(
        runtime_source.contains("record_live_provider_command_submitted")
            && runtime_source.contains("locked_at = NULL")
            && runtime_source.contains("reconciliation_status = 'awaiting_provider'")
            && command_executor_source.contains("execute_live_provider_command(&executable)")
            && command_executor_source.contains("record_live_provider_command_submitted")
            && command_executor_source.contains("provider_observed_event_reconciliation_required")
            && !command_executor_source.contains("completed_at ="),
        "WhatsApp live SDK success must record sanitized provider submission and continue waiting for provider-observed reconciliation"
    );
    assert!(
        runtime_source.contains("\"media_bytes\": \"excluded\"")
            && runtime_source.contains("\"media_key\": \"excluded\"")
            && runtime_source.contains("\"direct_path\": \"excluded\"")
            && runtime_source.contains("\"static_url\": \"excluded\"")
            && runtime_source.contains("media_key_sha256")
            && runtime_source.contains("native_md_wa_rs_sanitized_media_download_refs")
            && runtime_source.contains("native_md_store_media_download_ref")
            && runtime_source.contains("whatsapp_media_download_ref")
            && runtime_source.contains("host_vault_materialization")
            && runtime_source.contains("secret_ref_only_raw_refs_excluded")
            && runtime_source.contains("host_vault_only_not_postgres_events_logs_frontend")
            && runtime_source.contains("provider_request_id_matches_observed_media")
            && runtime_source
                .contains("provider_request_id_equals_observed_media_provider_message_id")
            && runtime_source
                .contains("\"accepted_event_kind\": \"signal.accepted.whatsapp.media\"")
            && runtime_source.contains("\"provider_observed_completion_target\"")
            && command_executor_source.contains("prepare_live_native_md_media_upload")
            && command_executor_source.contains("submitted_to_provider_awaiting_observed_evidence")
            && command_executor_source.contains("upload_media_sha256"),
        "WhatsApp live media upload/download-ref metadata must keep bytes/keys out of events while waiting for provider-observed reconciliation"
    );
}

#[test]
fn whatsapp_business_cloud_executor_is_smoke_gated_and_vault_token_only() {
    let root = repo_root();
    let runtime_source = read(root.join("backend/src/integrations/whatsapp/runtime/mod.rs"));
    let business_cloud_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/business_cloud.rs"));
    let contracts_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/contracts.rs"));
    let command_executor_source =
        read(root.join("backend/src/application/whatsapp_command_executor.rs"));
    let bootstrap_source = read(root.join("backend/src/application/bootstrap.rs"));
    let capabilities_source =
        read(root.join("backend/src/app/api_support/whatsapp_capability_catalog.rs"));

    assert!(
        contracts_source.contains("struct WhatsAppProviderApiAccessToken")
            && contracts_source.contains(".field(\"token\", &\"redacted\")")
            && contracts_source.contains("#[serde(skip_serializing)]")
            && contracts_source
                .contains("pub api_access_token: Option<WhatsAppProviderApiAccessToken>"),
        "Business Cloud access tokens must be in-memory, redacted and skipped during command serialization"
    );
    assert!(
        runtime_source.contains("claim_due_business_cloud_commands_for_execution")
            && runtime_source.contains("account.provider_kind = 'whatsapp_business_cloud'")
            && runtime_source
                .contains("account.config->>'provider_shape', '') = 'whatsapp_business_cloud'")
            && runtime_source.contains("account.config->>'runtime', '') = 'business_cloud_smoke'")
            && runtime_source.contains("business_cloud_live_smoke_enabled")
            && runtime_source.contains(
                "command.command_kind IN ('send_text', 'send_template', 'send_media', 'send_voice_note')"
            ),
        "Business Cloud command claim must be provider-shape scoped, smoke-gated and limited to the verified submission subset"
    );
    assert!(
        command_executor_source.contains("prepare_live_business_cloud_access_token")
            && command_executor_source.contains("vault.read_secret(&secret_ref)")
            && command_executor_source
                .contains("WhatsAppProviderApiAccessToken::new(secret_ref, token)")
            && command_executor_source.contains("prepare_live_business_cloud_media_upload")
            && command_executor_source.contains("business_cloud_media_blob_sha256_mismatch")
            && command_executor_source.contains("business_cloud_media_storage_kind_unsupported")
            && command_executor_source.contains("execute_due_live_business_cloud_commands")
            && bootstrap_source.contains("execute_due_live_business_cloud_commands"),
        "Business Cloud command worker must read tokens/media bytes from host vault/local blob boundaries and be wired into the scheduler"
    );
    assert!(
        business_cloud_source.contains("https://graph.facebook.com/{}/{}/messages")
            && business_cloud_source.contains("https://graph.facebook.com/{}/{}/media")
            && business_cloud_source.contains("access_token.expose_for_runtime()")
            && business_cloud_source.contains(".bearer_auth(access_token)")
            && business_cloud_source.contains("reqwest::multipart::Form::new()")
            && business_cloud_source.contains("\"send_template\"")
            && business_cloud_source.contains("\"send_media\"")
            && business_cloud_source.contains("\"send_voice_note\"")
            && business_cloud_source.contains("reqwest::header::RETRY_AFTER")
            && business_cloud_source.contains("business_cloud_rate_limited")
            && business_cloud_source.contains("provider_error_code")
            && business_cloud_source.contains("\"api_access_token\": \"excluded\"")
            && business_cloud_source.contains("\"template_components\": \"excluded\"")
            && business_cloud_source.contains("\"media_bytes\": \"excluded\"")
            && business_cloud_source.contains("\"media_filename\": \"excluded\"")
            && business_cloud_source.contains("\"media_caption\": \"excluded\"")
            && business_cloud_source.contains("\"raw_provider_payload\": \"excluded\"")
            && business_cloud_source.contains(
                "\"completion_rule\": \"provider_observed_event_reconciliation_required\""
            )
            && business_cloud_source.contains("\"provider_observed_completion_target\"")
            && business_cloud_source
                .contains("\"accepted_event_kind\": \"signal.accepted.whatsapp.receipt\"")
            && business_cloud_source.contains(
                "\"match_policy\": \"provider_request_id_equals_observed_receipt_provider_message_id\""
            )
            && business_cloud_source.contains("\"direct_domain_calls\": \"forbidden\""),
        "Business Cloud runtime submission must use official messages/media endpoint shapes and sanitize result metadata"
    );
    assert!(
        capabilities_source.contains("\"business.messages.send_text\"")
            && capabilities_source.contains("\"business.templates\"")
            && capabilities_source.contains("\"business.media_endpoints\"")
            && capabilities_source.contains("Business Cloud text send submission uses the official messages endpoint through the durable outbox"),
        "Business Cloud submissions must be exposed as Business Platform capabilities, not personal messages.send_text"
    );
}

#[test]
fn whatsapp_business_cloud_webhook_ingest_uses_event_spine_reconciliation() {
    let root = repo_root();
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let router_source = read(root.join("backend/src/app/router/routes/messaging.rs"));
    let client_models = read(root.join("backend/src/integrations/whatsapp/client/models.rs"));
    let platform_contract = read(root.join("backend/src/platform/communications.rs"));
    let business_cloud_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/business_cloud.rs"));
    let runtime_source = read(root.join("backend/src/integrations/whatsapp/runtime/mod.rs"));

    assert!(
        router_source
            .contains("/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks")
            && router_source.contains("get(get_whatsapp_runtime_bridge_business_cloud_webhook)")
            && router_source.contains("post_whatsapp_runtime_bridge_business_cloud_webhook"),
        "Business Cloud webhook verification/ingestion must be routed through the runtime-bridge namespace"
    );
    assert!(
        provider_handler.contains("WhatsAppBusinessCloudWebhookVerifyQuery")
            && provider_handler.contains("hub.verify_token")
            && provider_handler.contains("hub.challenge")
            && provider_handler.contains("verify_business_cloud_webhook_challenge_token")
            && provider_handler.contains("WhatsappBusinessCloudWebhookVerifyToken")
            && provider_handler.contains("whatsapp_business_cloud_webhook_verify_token_ref"),
        "Business Cloud webhook challenge verification must use account-scoped host-vault verify-token bindings"
    );
    assert!(
        provider_handler.contains("headers: HeaderMap")
            && provider_handler.contains("body: Bytes")
            && provider_handler.contains("X-Hub-Signature-256")
            && provider_handler.contains("verify_business_cloud_webhook_signature")
            && provider_handler.contains("HmacSha256")
            && provider_handler.contains("decode_sha256_hex")
            && provider_handler.contains("WhatsappBusinessCloudAppSecret")
            && provider_handler.contains("whatsapp_business_cloud_app_secret_ref"),
        "Business Cloud webhook POST must verify the raw request body with the vault-bound app secret before ingestion"
    );
    assert!(
        client_models.contains("pub app_secret: Option<String>")
            && client_models.contains("pub webhook_verify_token: Option<String>")
            && client_models.contains("\"app_secret\"")
            && client_models.contains("&self.app_secret.as_ref().map(|_| \"<redacted>\")")
            && client_models.contains("\"webhook_verify_token\"")
            && client_models
                .contains("&self.webhook_verify_token.as_ref().map(|_| \"<redacted>\")")
            && platform_contract.contains("WhatsappBusinessCloudAppSecret")
            && platform_contract.contains("WhatsappBusinessCloudWebhookVerifyToken"),
        "Business Cloud setup DTO and provider secret-purpose contract must keep webhook secrets explicit and redacted"
    );
    assert!(
        provider_handler.contains("post_whatsapp_runtime_bridge_business_cloud_webhook")
            && provider_handler.contains("business_cloud_webhook_message_request")
            && provider_handler.contains("business_cloud_webhook_receipt_request")
            && provider_handler.contains("ingest_runtime_bridge_message")
            && provider_handler.contains("ingest_runtime_bridge_receipt"),
        "Business Cloud webhook messages/statuses must normalize into existing runtime-bridge evidence contracts"
    );
    assert!(
        provider_handler.contains("unsupported_business_cloud_message")
            && provider_handler.contains("unsupported_business_cloud_status")
            && provider_handler.contains("ingest_runtime_bridge_runtime_event")
            && provider_handler.contains("\"raw_provider_payload\": \"excluded\"")
            && provider_handler.contains("\"api_access_token\": \"excluded\"")
            && provider_handler.contains("\"app_secret\": \"excluded\"")
            && provider_handler.contains("\"webhook_verify_token\": \"excluded\""),
        "Unsupported Business Cloud webhook items must become sanitized degraded runtime evidence, not be dropped or logged raw"
    );
    assert!(
        business_cloud_source.contains("\"provider_message_id\": provider_message_id.clone()")
            && business_cloud_source.contains("\"provider_request_id\": provider_message_id.clone()")
            && runtime_source.contains("provider_request_id_matches_observed_receipt")
            && runtime_source.contains("'send_text', 'send_template', 'reply', 'forward', 'send_media'")
            && runtime_source.contains("provider_state #>> '{business_cloud,provider_request_id}'")
            && runtime_source.contains(
                "provider_state #>> '{business_cloud,provider_observed_completion_target,provider_message_id}'"
            )
            && runtime_source.contains(
                "result_payload #>> '{provider_submission,provider_observed_completion_target,provider_message_id}'"
            )
            && business_cloud_source.contains(
                "\"completion_rule\": \"provider_observed_event_reconciliation_required\""
            ),
        "Business Cloud API submission must persist sanitized provider-observed receipt targets so webhook statuses can reconcile the command through event-spine evidence"
    );
}

#[test]
fn whatsapp_business_cloud_proxy_manifest_keeps_hermes_protected() {
    let root = repo_root();
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let router_source = read(root.join("backend/src/app/router/routes/messaging.rs"));
    let public_routes = read(root.join("backend/src/app/router/routes/public.rs"));
    let cargo_toml = read(root.join("backend/Cargo.toml"));
    let dockerfile = read(root.join("docker/Dockerfile"));
    let docker_compose = read(root.join("docker/docker-compose.yml"));
    let docker_env_example = read(root.join("docker/.env.example"));
    let makefile = read(root.join("Makefile"));
    let edge_proxy =
        read(root.join("backend/src/bin/hermes_whatsapp_business_cloud_edge_proxy.rs"));
    let local_api_secret_env = concat!("HERMES_LOCAL", "_API_SECRET");

    assert!(
        router_source
            .contains("/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest")
            && router_source
                .contains("get(get_whatsapp_runtime_bridge_business_cloud_proxy_manifest)"),
        "Business Cloud edge/proxy contract must be exposed as a protected runtime-bridge manifest"
    );
    assert!(
        !public_routes.contains("runtime-bridge/business-cloud/webhooks")
            && !public_routes.contains("runtime-bridge/business-cloud/proxy-manifest"),
        "Business Cloud webhook and proxy manifest must not be registered as unauthenticated public Hermes routes"
    );
    assert!(
        provider_handler.contains("WhatsAppBusinessCloudProxyManifestResponse")
            && provider_handler.contains("hermes_direct_public_endpoint: false")
            && provider_handler.contains("edge_proxy_required: true")
            && provider_handler.contains("local_auth_header: \"X-Hermes-Secret\"")
            && provider_handler
                .contains("post_raw_body_must_be_forwarded_byte_for_byte_before_json_parse")
            && provider_handler.contains("forward_exact_raw_body_do_not_inject_or_rewrite_payload")
            && provider_handler
                .contains("host_vault_whatsapp_business_cloud_app_secret_hmac_sha256")
            && provider_handler.contains("host_vault_whatsapp_business_cloud_webhook_verify_token")
            && provider_handler.contains("secret_ref_bindings_only_secret_values_excluded"),
        "Business Cloud proxy manifest must document the local-auth, raw-body, HMAC and redaction contract"
    );
    assert!(
        provider_handler.contains("webhook_proxy_ready")
            && provider_handler.contains("command_submission_ready")
            && provider_handler.contains("missing_whatsapp_business_cloud_app_secret_binding")
            && provider_handler
                .contains("missing_whatsapp_business_cloud_webhook_verify_token_binding")
            && provider_handler.contains("business_cloud_live_smoke_not_enabled"),
        "Business Cloud proxy manifest must distinguish webhook readiness from smoke-gated command readiness"
    );

    let manifest_handler = source_between(
        &provider_handler,
        "pub(crate) async fn get_whatsapp_runtime_bridge_business_cloud_proxy_manifest",
        "pub(crate) async fn get_whatsapp_runtime_bridge_business_cloud_webhook",
    );
    let manifest_accounts = source_between(
        &provider_handler,
        "async fn business_cloud_proxy_manifest_accounts",
        "fn business_cloud_proxy_account_blockers",
    );
    assert!(
        !manifest_handler.contains("read_secret") && !manifest_accounts.contains("read_secret"),
        "Business Cloud proxy manifest may report secret bindings but must not read host-vault secret values"
    );
    assert!(
        cargo_toml.contains("hermes-whatsapp-business-cloud-edge-proxy")
            && edge_proxy
                .contains("PUBLIC_WEBHOOK_PATH: &str = \"/webhooks/whatsapp/business-cloud\"")
            && edge_proxy.contains("PROTECTED_HERMES_WEBHOOK_PATH")
            && edge_proxy.contains("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET")
            && edge_proxy.contains(local_api_secret_env)
            && edge_proxy.contains("HERMES_SECRET_HEADER")
            && edge_proxy.contains("BUSINESS_CLOUD_SIGNATURE_HEADER")
            && edge_proxy.contains("RawQuery")
            && edge_proxy.contains(".body(body)")
            && edge_proxy.contains("hermes_url(PROTECTED_HERMES_MANIFEST_PATH, None, false)")
            && edge_proxy
                .contains("hermes_url(PROTECTED_HERMES_WEBHOOK_PATH, raw_query.as_deref(), true)")
            && edge_proxy.contains("hermes_url(PROTECTED_HERMES_WEBHOOK_PATH, None, false)")
            && edge_proxy.contains("readyz_checks_manifest_without_account_scoping")
            && edge_proxy.contains("post_webhook_forwards_raw_body_signature_and_no_account_query")
            && edge_proxy.contains("post_body_is_not_parsed_or_rewritten_by_edge_proxy"),
        "Business Cloud edge proxy binary must expose the public webhook path and forward to protected Hermes with local auth, raw GET query/body/signature and account scoping only where intended"
    );
    assert!(
        !edge_proxy.contains("serde_json::from_slice")
            && !edge_proxy.contains("serde_json::from_str")
            && !edge_proxy.contains("Json<Value>")
            && !edge_proxy.contains("read_secret"),
        "Business Cloud edge proxy must not parse webhook JSON or read vault secret values; Hermes performs verified ingestion"
    );
    assert!(
        dockerfile.contains("AS whatsapp-business-cloud-edge-proxy")
            && dockerfile.contains("--bin hermes-whatsapp-business-cloud-edge-proxy")
            && dockerfile.contains("/usr/local/bin/hermes-whatsapp-business-cloud-edge-proxy")
            && dockerfile.contains("EXPOSE 8787")
            && docker_compose.contains("whatsapp-business-cloud-edge-proxy:")
            && docker_compose.contains("profiles:")
            && docker_compose.contains("whatsapp-business-cloud-edge")
            && docker_compose.contains("target: whatsapp-business-cloud-edge-proxy")
            && docker_compose.contains("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR: 0.0.0.0:8787")
            && docker_compose.contains("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL")
            && docker_compose.contains(local_api_secret_env)
            && docker_compose.contains("host.docker.internal:host-gateway")
            && docker_compose.contains("curl -fsS http://127.0.0.1:8787/healthz")
            && docker_env_example.contains("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND=127.0.0.1")
            && docker_env_example.contains("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT=8787")
            && docker_env_example.contains(
                "HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL=http://host.docker.internal:8080"
            )
            && makefile.contains("whatsapp-business-cloud-edge-config")
            && makefile.contains("whatsapp-business-cloud-edge-up")
            && makefile.contains("whatsapp-business-cloud-edge-stop")
            && makefile.contains("whatsapp-business-cloud-edge-logs"),
        "Business Cloud edge proxy must be packaged as an opt-in compose profile with non-secret env placeholders and Makefile entry points"
    );
}

struct SignalHubFixture {
    raw_record_kind: Option<&'static str>,
    signal_kind: &'static str,
    matrix_label: &'static str,
    realtime_event_constant: &'static str,
}

fn signal_hub_fixtures() -> Vec<SignalHubFixture> {
    vec![
        SignalHubFixture {
            raw_record_kind: None,
            signal_kind: "message",
            matrix_label: "whatsapp_message",
            realtime_event_constant: "MESSAGE_CREATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_message_update"),
            signal_kind: "message_update",
            matrix_label: "whatsapp_message_update",
            realtime_event_constant: "MESSAGE_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_message_delete"),
            signal_kind: "message_delete",
            matrix_label: "whatsapp_message_delete",
            realtime_event_constant: "MESSAGE_DELETED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_receipt"),
            signal_kind: "receipt",
            matrix_label: "whatsapp_receipt",
            realtime_event_constant: "RECEIPT_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_reaction"),
            signal_kind: "reaction",
            matrix_label: "whatsapp_reaction",
            realtime_event_constant: "REACTION_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_dialog"),
            signal_kind: "dialog",
            matrix_label: "whatsapp_dialog",
            realtime_event_constant: "DIALOG_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_participant"),
            signal_kind: "participant",
            matrix_label: "whatsapp_participant",
            realtime_event_constant: "PARTICIPANT_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_presence"),
            signal_kind: "presence",
            matrix_label: "whatsapp_presence",
            realtime_event_constant: "PRESENCE_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_call"),
            signal_kind: "call_metadata",
            matrix_label: "whatsapp_call_metadata",
            realtime_event_constant: "CALL_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status"),
            signal_kind: "status",
            matrix_label: "whatsapp_status",
            realtime_event_constant: "STATUS_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status_view"),
            signal_kind: "status_view",
            matrix_label: "whatsapp_status_view",
            realtime_event_constant: "STATUS_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status_delete"),
            signal_kind: "status_delete",
            matrix_label: "whatsapp_status_delete",
            realtime_event_constant: "STATUS_DELETED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_media"),
            signal_kind: "media",
            matrix_label: "whatsapp_media",
            realtime_event_constant: "MEDIA_DOWNLOAD_COMPLETED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_runtime_event"),
            signal_kind: "runtime_event",
            matrix_label: "whatsapp_runtime_event",
            realtime_event_constant: "RUNTIME_EVENT",
        },
    ]
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("source marker missing: {start}"));
    let relative_end_index = source[start_index..]
        .find(end)
        .unwrap_or_else(|| panic!("source marker missing after {start}: {end}"));
    &source[start_index..start_index + relative_end_index]
}

fn read_all_sources(root: PathBuf) -> String {
    let mut output = String::new();
    collect_sources(&root, &mut output);
    output
}

fn collect_sources(path: &Path, output: &mut String) {
    if path.is_file() {
        if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs")
        ) {
            output.push_str(&read(path.to_path_buf()));
            output.push('\n');
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_sources(&entry.path(), output);
    }
}
