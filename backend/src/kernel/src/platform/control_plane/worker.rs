//! Classifies private control-plane workers without weakening critical recovery.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;

const RESTART_BACKOFF: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkerClassV1 {
    Critical,
    Restartable,
}

pub(super) struct WorkerCompletionV1 {
    pub(super) class: WorkerClassV1,
    pub(super) label: &'static str,
    pub(super) result: Result<(), String>,
}

pub(super) fn spawn<F>(
    completed: Sender<WorkerCompletionV1>,
    class: WorkerClassV1,
    label: &'static str,
    shutdown_requested: Arc<AtomicBool>,
    serve: F,
) -> std::thread::JoinHandle<()>
where
    F: Fn(Arc<AtomicBool>) -> Result<(), String> + Send + Sync + 'static,
{
    std::thread::spawn(move || {
        loop {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                serve(Arc::clone(&shutdown_requested))
            }))
            .unwrap_or_else(|_| Err("control-plane worker panicked".to_owned()));
            if shutdown_requested.load(Ordering::Acquire) {
                return;
            }
            if class == WorkerClassV1::Critical {
                let _ = completed.send(WorkerCompletionV1 {
                    class,
                    label,
                    result,
                });
                return;
            }
            let result = result.and(Err("restartable worker stopped unexpectedly".to_owned()));
            let _ = completed.send(WorkerCompletionV1 {
                class,
                label,
                result,
            });
            std::thread::sleep(RESTART_BACKOFF);
        }
    })
}
