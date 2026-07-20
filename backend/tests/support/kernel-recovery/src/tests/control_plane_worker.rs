use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use crate::control_plane_worker::{WorkerClassV1, spawn};

#[test]
fn restartable_worker_failure_keeps_the_kernel_shutdown_signal_clear() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();
    let worker_shutdown = Arc::clone(&shutdown);
    let worker_attempts = Arc::clone(&attempts);
    let worker = spawn(
        sender,
        WorkerClassV1::Restartable,
        "optional-worker",
        worker_shutdown,
        move |_| {
            worker_attempts.fetch_add(1, Ordering::AcqRel);
            Err("transient failure".to_owned())
        },
    );
    let completion = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("restartable completion");
    assert_eq!(completion.class, WorkerClassV1::Restartable);
    assert!(completion.result.is_err());
    assert!(!shutdown.load(Ordering::Acquire));
    assert_eq!(attempts.load(Ordering::Acquire), 1);
    shutdown.store(true, Ordering::Release);
    worker
        .join()
        .expect("restartable worker joins after shutdown");
}

#[test]
fn critical_worker_reports_failure_before_supervisor_requests_shutdown() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel();
    let worker = spawn(
        sender,
        WorkerClassV1::Critical,
        "recovery",
        Arc::clone(&shutdown),
        move |_| Err("critical failure".to_owned()),
    );
    let completion = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("critical completion");
    assert_eq!(completion.class, WorkerClassV1::Critical);
    assert!(completion.result.is_err());
    assert!(!shutdown.load(Ordering::Acquire));
    worker.join().expect("critical worker joins");
}
