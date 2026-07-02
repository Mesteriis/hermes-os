use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

mod whatsapp_companion;
mod yandex_telemost_companion;

const DEFAULT_LOCAL_API_SECRET: &str = "change-me-local-api-secret";

/// Resolution order: explicit runtime env, then the per-build random secret
/// baked in by `scripts/build.sh` (HERMES_BUNDLED_LOCAL_API_SECRET), then the
/// shared local-development fallback.
pub(crate) fn local_api_secret() -> String {
    if let Ok(value) = std::env::var("HERMES_LOCAL_API_SECRET") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    option_env!("HERMES_BUNDLED_LOCAL_API_SECRET")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| DEFAULT_LOCAL_API_SECRET.to_owned())
}

#[derive(Default)]
struct BackendSidecar {
    child: Mutex<Option<CommandChild>>,
}

impl Drop for BackendSidecar {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            if let Some(child) = child.take() {
                let _ = child.kill();
            }
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            whatsapp_companion::open_whatsapp_web_companion,
            whatsapp_companion::whatsapp_web_companion_manifest,
            whatsapp_companion::whatsapp_web_companion_relay_observation,
            yandex_telemost_companion::open_yandex_telemost_companion,
            yandex_telemost_companion::yandex_telemost_companion_manifest,
            yandex_telemost_companion::yandex_telemost_prepare_audio_device,
            yandex_telemost_companion::yandex_telemost_recording_start,
            yandex_telemost_companion::yandex_telemost_recording_stop,
            yandex_telemost_companion::yandex_telemost_speaker_timeline_append,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(BackendSidecar::default());
            app.manage(yandex_telemost_companion::TelemostLocalRecorder::default());
            if !cfg!(debug_assertions) {
                start_backend_sidecar(app.handle())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_backend_sidecar<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("HERMES_DISABLE_BACKEND_SIDECAR").is_some() {
        log::info!("Hermes backend sidecar disabled by HERMES_DISABLE_BACKEND_SIDECAR");
        return Ok(());
    }

    let mut command = app
        .shell()
        .sidecar("hermes-hub-backend")?
        .env("HERMES_HTTP_ADDR", "127.0.0.1:8080")
        .env("HERMES_LOCAL_API_SECRET", local_api_secret());

    for key in [
        "DATABASE_URL",
        "HERMES_SECRET_VAULT_KEY",
        "HERMES_OLLAMA_BASE_URL",
        "HERMES_OLLAMA_CHAT_MODEL",
        "HERMES_OLLAMA_EMBED_MODEL",
        "HERMES_OLLAMA_TIMEOUT_SECONDS",
        "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
        "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH",
        "HERMES_GOOGLE_OAUTH_CLIENT_ID",
        "HERMES_GOOGLE_OAUTH_CLIENT_SECRET",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command = command.env(key, value);
        }
    }
    if let Some(value) = std::env::var_os("HERMES_TELEGRAM_API_ID")
        .or_else(|| option_env!("HERMES_BUNDLED_TELEGRAM_API_ID").map(std::ffi::OsString::from))
    {
        command = command.env("HERMES_TELEGRAM_API_ID", value);
    }
    if let Some(value) = std::env::var_os("HERMES_TELEGRAM_API_HASH")
        .or_else(|| option_env!("HERMES_BUNDLED_TELEGRAM_API_HASH").map(std::ffi::OsString::from))
    {
        command = command.env("HERMES_TELEGRAM_API_HASH", value);
    }

    if std::env::var_os("HERMES_TDJSON_PATH").is_none() {
        if let Some(tdjson_path) = bundled_tdjson_path(app) {
            command = command.env("HERMES_TDJSON_PATH", tdjson_path);
        }
    }
    if std::env::var_os("HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH").is_none()
        && std::env::var_os("HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON").is_none()
        && std::env::var_os("HERMES_GOOGLE_OAUTH_CLIENT_ID").is_none()
    {
        if let Some(google_oauth_client_config_path) = bundled_google_oauth_client_config_path(app)
        {
            command = command.env(
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH",
                google_oauth_client_config_path,
            );
        }
    }

    let (mut events, child) = command.spawn()?;
    let pid = child.pid();
    app.state::<BackendSidecar>()
        .child
        .lock()
        .map_err(|_| std::io::Error::other("backend sidecar state lock poisoned"))?
        .replace(child);

    tauri::async_runtime::spawn(async move {
        log::info!("Hermes backend sidecar started with pid {pid}");
        while let Some(event) = events.recv().await {
            match event {
                CommandEvent::Stdout(line) => log_sidecar_line(log::Level::Info, &line),
                CommandEvent::Stderr(line) => log_sidecar_line(log::Level::Warn, &line),
                CommandEvent::Error(error) => {
                    log::error!("Hermes backend sidecar event error: {error}");
                }
                CommandEvent::Terminated(payload) => {
                    log::warn!(
                        "Hermes backend sidecar terminated: code={:?} signal={:?}",
                        payload.code,
                        payload.signal
                    );
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn log_sidecar_line(level: log::Level, bytes: &[u8]) {
    let line = String::from_utf8_lossy(bytes).trim().to_owned();
    if line.is_empty() {
        return;
    }
    log::log!(level, "Hermes backend sidecar: {line}");
}

fn bundled_tdjson_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    let tdlib_dir = resource_dir.join("tdlib");
    let platform_path = tdlib_dir
        .join(tdlib_platform_dir())
        .join(tdlib_library_file_name());
    if platform_path.is_file() {
        return Some(platform_path);
    }

    let universal_path = tdlib_dir
        .join("macos-universal")
        .join(tdlib_library_file_name());
    universal_path.is_file().then_some(universal_path)
}

fn bundled_google_oauth_client_config_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    let client_config_path = resource_dir.join("google-oauth").join("client_secret.json");
    client_config_path.is_file().then_some(client_config_path)
}

fn tdlib_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "macos-arm64";
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "macos-x64";
    }
    #[allow(unreachable_code)]
    "unknown"
}

fn tdlib_library_file_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "libtdjson.dylib";
    }
    #[allow(unreachable_code)]
    "libtdjson"
}
