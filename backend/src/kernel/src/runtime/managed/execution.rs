//! Runs one preflighted staged executable with bounded failure retries.

use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::distribution::staged_artifact::StagedNativeArtifact;

const MAX_ATTEMPTS: u8 = 8;
const MAX_RUNTIME_SECONDS: u64 = 300;

pub struct ManagedChildExecutionPolicy {
    max_attempts: u8,
    max_runtime: Duration,
}

impl ManagedChildExecutionPolicy {
    pub fn new(max_attempts: u8, max_runtime: Duration) -> Result<Self, String> {
        if !(1..=MAX_ATTEMPTS).contains(&max_attempts) {
            return Err(format!(
                "managed child attempts must be between 1 and {MAX_ATTEMPTS}"
            ));
        }
        if max_runtime.is_zero() || max_runtime > Duration::from_secs(MAX_RUNTIME_SECONDS) {
            return Err(format!(
                "managed child runtime must be between 1 second and {MAX_RUNTIME_SECONDS} seconds"
            ));
        }
        Ok(Self {
            max_attempts,
            max_runtime,
        })
    }

    #[must_use]
    pub fn max_attempts(&self) -> u8 {
        self.max_attempts
    }

    #[must_use]
    pub fn max_runtime(&self) -> Duration {
        self.max_runtime
    }
}

#[derive(Debug)]
pub struct ManagedChildExecutionResult {
    attempts: u8,
    exit_code: i32,
}

impl ManagedChildExecutionResult {
    pub(crate) fn succeeded(attempts: u8, exit_code: i32) -> Self {
        Self {
            attempts,
            exit_code,
        }
    }

    #[must_use]
    pub fn attempts(&self) -> u8 {
        self.attempts
    }

    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

pub fn run(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    policy: &ManagedChildExecutionPolicy,
) -> Result<ManagedChildExecutionResult, String> {
    for attempt in 1..=policy.max_attempts {
        let mut child = spawn(staged_executable, arguments, Stdio::null())?;
        let status = wait(&mut child, policy.max_runtime)?;
        if status.success() {
            return Ok(ManagedChildExecutionResult::succeeded(
                attempt,
                status.code().unwrap_or(0),
            ));
        }
    }
    Err("managed child exhausted its bounded restart attempts".to_owned())
}

pub fn spawn(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    stdin: Stdio,
) -> Result<Child, String> {
    ensure_staged_executable(staged_executable.path())?;
    let stderr = if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
    let mut command = Command::new(staged_executable.path());
    command
        .args(arguments)
        .stdin(stdin)
        .stdout(Stdio::null())
        .stderr(stderr)
        .env_clear();
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        command.env("HERMES_DEVELOPER_VERBOSE", "1");
    }
    command
        .spawn()
        .map_err(|error| format!("managed child could not start: {error}"))
}

pub fn wait(child: &mut Child, max_runtime: Duration) -> Result<ExitStatus, String> {
    wait_with_shutdown(child, Some(max_runtime), None)
}

pub fn wait_until_shutdown(
    child: &mut Child,
    shutdown_requested: &AtomicBool,
) -> Result<ExitStatus, String> {
    wait_with_shutdown(child, None, Some(shutdown_requested))
}

fn wait_with_shutdown(
    child: &mut Child,
    max_runtime: Option<Duration>,
    shutdown_requested: Option<&AtomicBool>,
) -> Result<ExitStatus, String> {
    let deadline = max_runtime.map(|duration| Instant::now() + duration);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status);
        }
        if shutdown_requested.is_some_and(|requested| requested.load(Ordering::Acquire)) {
            terminate(child)?;
            return Err("managed child stopped by Kernel shutdown".to_owned());
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            terminate(child)?;
            return Err("managed child exceeded its bounded runtime".to_owned());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

pub fn terminate(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        child.kill().map_err(|error| error.to_string())?;
        let _ = child.wait().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn ensure_staged_executable(path: &std::path::Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("managed child executable path must be absolute".to_owned());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o100 == 0
    {
        return Err(
            "managed child executable must be a regular executable non-symlink file".to_owned(),
        );
    }
    Ok(())
}
