const APP_COMMANDS: &[&str] = &[
    "open_whatsapp_web_companion",
    "whatsapp_web_companion_manifest",
    "whatsapp_web_companion_relay_observation",
    "open_yandex_telemost_companion",
    "yandex_telemost_companion_manifest",
    "yandex_telemost_prepare_audio_device",
    "yandex_telemost_recording_start",
    "yandex_telemost_recording_stop",
    "yandex_telemost_speaker_timeline_append",
];

fn main() {
    // The per-build random API secret is embedded via option_env!, so the
    // crate must be recompiled when the build script provides a new value.
    println!("cargo:rerun-if-env-changed=HERMES_BUNDLED_LOCAL_API_SECRET");
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run tauri build script")
}
