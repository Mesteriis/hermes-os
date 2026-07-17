//! Couples a fresh inherited control FD to each bounded managed-child attempt.

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::runtime::lifecycle::control::{
    self as managed_runtime_control, ManagedRuntimeExpectation,
};
use crate::runtime::managed::execution::{
    self as bounded_managed_child_execution, ManagedChildExecutionPolicy,
    ManagedChildExecutionResult,
};
use std::process::{Child, ExitStatus};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

pub fn run(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
) -> Result<ManagedChildExecutionResult, String> {
    run_with_wait(
        staged_executable,
        arguments,
        expectation,
        policy,
        |child, _| bounded_managed_child_execution::wait(child, policy.max_runtime()),
    )
}

pub fn run_until_shutdown(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
    shutdown_requested: &AtomicBool,
    stop_requested: &AtomicBool,
    relay_requests: &Receiver<managed_runtime_control::ManagedRuntimeRelayRequest>,
) -> Result<ManagedChildExecutionResult, String> {
    run_with_wait(
        staged_executable,
        arguments,
        expectation,
        policy,
        |child, channel| {
            wait_until_shutdown_with_relay(
                child,
                channel,
                shutdown_requested,
                stop_requested,
                relay_requests,
            )
        },
    )
}

fn run_with_wait<F>(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
    mut wait: F,
) -> Result<ManagedChildExecutionResult, String>
where
    F: FnMut(&mut Child, &mut std::os::unix::net::UnixStream) -> Result<ExitStatus, String>,
{
    for attempt in 1..=policy.max_attempts() {
        let (kernel_end, child_stdin) = managed_runtime_control::create_inherited_channel()?;
        let mut child =
            bounded_managed_child_execution::spawn(staged_executable, arguments, child_stdin)?;
        let mut control_channel =
            match managed_runtime_control::establish_channel(kernel_end, expectation) {
                Ok(channel) => channel,
                Err(error) => {
                    let _ = bounded_managed_child_execution::terminate(&mut child);
                    if attempt == policy.max_attempts() {
                        return Err(error);
                    }
                    continue;
                }
            };
        let status = wait(&mut child, &mut control_channel)?;
        if status.success() {
            return Ok(ManagedChildExecutionResult::succeeded(
                attempt,
                status.code().unwrap_or(0),
            ));
        }
    }
    Err("managed child exhausted its bounded restart attempts".to_owned())
}

fn wait_until_shutdown_with_relay(
    child: &mut Child,
    channel: &mut std::os::unix::net::UnixStream,
    shutdown_requested: &AtomicBool,
    stop_requested: &AtomicBool,
    relay_requests: &Receiver<managed_runtime_control::ManagedRuntimeRelayRequest>,
) -> Result<ExitStatus, String> {
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status);
        }
        if shutdown_requested.load(std::sync::atomic::Ordering::Acquire)
            || stop_requested.load(std::sync::atomic::Ordering::Acquire)
        {
            bounded_managed_child_execution::terminate(child)?;
            return Err("managed child stopped by Kernel shutdown".to_owned());
        }
        match relay_requests.recv_timeout(Duration::from_millis(25)) {
            Ok(request) => request.dispatch(channel),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                bounded_managed_child_execution::terminate(child)?;
                return Err("managed runtime relay was disconnected".to_owned());
            }
        }
    }
}
