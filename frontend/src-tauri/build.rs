const APP_COMMANDS: &[&str] = &[
    "owner_vault_provisioning_host_start",
    "owner_vault_provisioning_host_seal",
    "owner_vault_provisioning_host_open_receipt",
    "owner_vault_provisioning_host_cancel",
    "open_whatsapp_web_companion",
    "hide_whatsapp_web_companion",
    "start_hidden_whatsapp_webview",
    "whatsapp_web_companion_manifest",
    "connect_whatsapp_runtime_bridge",
    "whatsapp_web_companion_relay_runtime_state",
    "open_yandex_telemost_companion",
    "yandex_telemost_companion_manifest",
    "desktop_call_recording_host_connect",
    "desktop_call_recording_host_disconnect",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run tauri build script")
}
