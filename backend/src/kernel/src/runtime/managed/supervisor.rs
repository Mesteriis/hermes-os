//! Couples a fresh inherited control FD to each bounded managed-child attempt.

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::runtime::lifecycle::control::{
    self as managed_runtime_control, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation, ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::managed::execution::{
    self as bounded_managed_child_execution, ManagedChildExecutionPolicy,
    ManagedChildExecutionResult,
};
use std::process::{Child, ExitStatus};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};

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
    vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
    event_credential_handler: Option<&dyn ManagedRuntimeEventCredentialHandler>,
    ready_sender: &SyncSender<Result<(), String>>,
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
                expectation,
                shutdown_requested,
                stop_requested,
                relay_requests,
                vault_route_handler,
                event_credential_handler,
                ready_sender,
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
    expectation: &ManagedRuntimeExpectation,
    shutdown_requested: &AtomicBool,
    stop_requested: &AtomicBool,
    relay_requests: &Receiver<managed_runtime_control::ManagedRuntimeRelayRequest>,
    vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
    event_credential_handler: Option<&dyn ManagedRuntimeEventCredentialHandler>,
    ready_sender: &SyncSender<Result<(), String>>,
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
        if let Some(ready) = managed_runtime_control::inbound::try_receive_ready(channel)? {
            if !expectation.matches_ready(&ready) {
                let _ =
                    ready_sender.try_send(Err("managed runtime ready signal is stale".to_owned()));
                return Err("managed runtime ready signal is stale".to_owned());
            }
            let _ = ready_sender.try_send(Ok(()));
            continue;
        }
        match process_typed_requests(
            channel,
            expectation,
            vault_route_handler,
            event_credential_handler,
        ) {
            Ok(true) => continue,
            Ok(false) => {}
            Err(_) => return terminal_status_after_control_close(child),
        }
        match relay_requests.recv_timeout(Duration::from_millis(25)) {
            Ok(request) => request.dispatch(channel, expectation, vault_route_handler),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                bounded_managed_child_execution::terminate(child)?;
                return Err("managed runtime relay was disconnected".to_owned());
            }
        }
    }
}

fn process_typed_requests(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
    event_credential_handler: Option<&dyn ManagedRuntimeEventCredentialHandler>,
) -> Result<bool, String> {
    if let Some(route) = managed_runtime_control::inbound::try_receive_vault_route(channel)? {
        let result = vault_route_handler
            .ok_or_else(|| "managed runtime Vault route is not available".to_owned())?
            .route_vault_ciphertext(expectation, route);
        managed_runtime_control::inbound::respond_vault_route(channel, result)?;
        return Ok(true);
    }
    if let Some(request) = managed_runtime_control::inbound::try_receive_event_credential(channel)?
    {
        dispatch_event_credential(channel, expectation, event_credential_handler, request)?;
        return Ok(true);
    }
    Ok(false)
}

fn dispatch_event_credential(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeEventCredentialHandler>,
    request: hermes_runtime_protocol::v1::ManagedRuntimeEventCredentialRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime Event credential route is not available".to_owned())?
        .issue_event_credential(expectation, request);
    managed_runtime_control::inbound::respond_event_credential(channel, result)
}

fn terminal_status_after_control_close(child: &mut Child) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = child.try_wait().map_err(|failure| failure.to_string())? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            child.kill().map_err(|failure| failure.to_string())?;
            return child.wait().map_err(|failure| failure.to_string());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
