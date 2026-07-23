use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

#[cfg(feature = "whatsapp-host-webview")]
mod whatsapp_companion;
#[cfg(feature = "telemost-host-companion")]
mod yandex_telemost_companion;

#[derive(Default)]
struct KernelSidecar {
    child: Mutex<Option<CommandChild>>,
    stopping: AtomicBool,
}

const MAX_KERNEL_RESTARTS: u8 = 3;

impl KernelSidecar {
    fn stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }
}

impl Drop for KernelSidecar {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        if let Ok(mut child) = self.child.lock() {
            if let Some(child) = child.take() {
                let _ = child.kill();
            }
        }
    }
}

pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());
    #[cfg(feature = "whatsapp-host-webview")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        whatsapp_companion::start_hidden_whatsapp_webview,
        whatsapp_companion::whatsapp_web_companion_manifest,
        whatsapp_companion::open_whatsapp_web_companion,
        whatsapp_companion::hide_whatsapp_web_companion,
        whatsapp_companion::connect_whatsapp_runtime_bridge,
        whatsapp_companion::whatsapp_web_companion_relay_runtime_state,
    ]);
    #[cfg(feature = "telemost-host-companion")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        yandex_telemost_companion::open_yandex_telemost_companion,
        yandex_telemost_companion::yandex_telemost_companion_manifest,
        yandex_telemost_companion::yandex_telemost_prepare_audio_device,
        yandex_telemost_companion::yandex_telemost_recording_start,
        yandex_telemost_companion::yandex_telemost_recording_stop,
        yandex_telemost_companion::yandex_telemost_speaker_timeline_append,
    ]);

    builder
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(KernelSidecar::default());
            #[cfg(feature = "whatsapp-host-webview")]
            app.manage(whatsapp_companion::WhatsAppHostRoutes::default());
            #[cfg(feature = "telemost-host-companion")]
            app.manage(yandex_telemost_companion::TelemostLocalRecorder::default());
            if !cfg!(debug_assertions) {
                start_kernel_sidecar(app.handle().clone(), 0)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_kernel_sidecar<R: Runtime>(
    app: AppHandle<R>,
    restart_attempt: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    if app.state::<KernelSidecar>().stopping() {
        return Ok(());
    }

    let command = app
        .shell()
        .sidecar("hermes-kernel")?
        .env_clear()
        .arg("--data-dir")
        .arg(app.path().app_local_data_dir()?)
        .arg("serve");

    let (mut events, child) = command.spawn()?;
    let pid = child.pid();
    app.state::<KernelSidecar>()
        .child
        .lock()
        .map_err(|_| std::io::Error::other("kernel sidecar state lock poisoned"))?
        .replace(child);

    let app_for_events = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Hermes Kernel sidecar started with pid {pid}");
        while let Some(event) = events.recv().await {
            match event {
                CommandEvent::Stdout(_) | CommandEvent::Stderr(_) => {
                    log::debug!("Hermes Kernel sidecar emitted output (suppressed)");
                }
                CommandEvent::Error(_) => log::error!("Hermes Kernel sidecar event error"),
                CommandEvent::Terminated(payload) => {
                    log::warn!(
                        "Hermes Kernel sidecar terminated: code={:?} signal={:?}",
                        payload.code,
                        payload.signal
                    );
                    if !app_for_events.state::<KernelSidecar>().stopping()
                        && restart_attempt < MAX_KERNEL_RESTARTS
                    {
                        if let Err(error) =
                            start_kernel_sidecar(app_for_events.clone(), restart_attempt + 1)
                        {
                            log::error!("Hermes Kernel bounded restart failed: {error}");
                        }
                    }
                    return;
                }
                _ => {}
            }
        }
    });

    Ok(())
}
