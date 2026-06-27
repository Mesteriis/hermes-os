const APP_COMMANDS: &[&str] = &[
    "open_whatsapp_web_companion",
    "whatsapp_web_companion_manifest",
    "whatsapp_web_companion_relay_observation",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run tauri build script")
}
