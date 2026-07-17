//! Shared bounded shutdown signal for one Kernel process.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn install() -> Result<Arc<AtomicBool>, String> {
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::signal::SIGINT,
        signal_hook::consts::signal::SIGTERM,
    ] {
        signal_hook::flag::register(signal, Arc::clone(&shutdown_requested))
            .map_err(|error| error.to_string())?;
    }
    Ok(shutdown_requested)
}
