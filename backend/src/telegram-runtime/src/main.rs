//! Telegram integration process composition root.

use hermes_telegram_api::TelegramRuntimeState;
use hermes_telegram_tdlib::TdJsonLibrary;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "status" => {
            let tdlib_path = std::env::var_os("HERMES_TELEGRAM_TDJSON_PATH").map(PathBuf::from);
            let tdlib_available = TdJsonLibrary::load(tdlib_path.as_deref()).is_ok();
            let state = if tdlib_available {
                TelegramRuntimeState::Stopped
            } else {
                TelegramRuntimeState::Blocked
            };
            println!(
                "telegram_runtime state={:?} tdlib_available={tdlib_available}",
                state
            );
            Ok(())
        }
        "probe" => {
            let tdlib_path = std::env::var_os("HERMES_TELEGRAM_TDJSON_PATH").map(PathBuf::from);
            TdJsonLibrary::load(tdlib_path.as_deref())
                .map(|_| println!("telegram_runtime tdlib_probe=ready"))
                .map_err(|error| format!("telegram TDLib probe failed: {error:?}"))
        }
        "start" | "stop" => Err(format!(
            "telegram runtime requires an admitted account and TDLib transport: command={command}"
        )),
        other => Err(format!("telegram runtime command is unavailable: {other}")),
    }
}
