//! Owner-authorized owner-control commands for Mail and Communications runtimes.

use std::path::Path;
use std::process::Command;

use hermes_gateway_protocol::v1::{
    ExecuteCommunicationsRuntimeOwnerCommandRequestV1,
    ExecuteCommunicationsRuntimeOwnerCommandResponseV1, ExecuteMailRuntimeOwnerCommandRequestV1,
    ExecuteMailRuntimeOwnerCommandResponseV1, ExecuteWhatsAppRuntimeOwnerCommandRequestV1,
    ExecuteWhatsAppRuntimeOwnerCommandResponseV1,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

use super::{OwnerControlSessions, OwnerResult};

const MAIL_RUNTIME_PROCESS_ID: &str = "hermes-mail-runtime";
const COMMUNICATIONS_RUNTIME_PROCESS_ID: &str = "hermes-communications-runtime";
const WHATSAPP_RUNTIME_PROCESS_ID: &str = "hermes-whatsapp-runtime";

const ALLOWED_MAIL_COMMANDS: &[&str] = &[
    "status",
    "begin",
    "sync",
    "complete",
    "get-connection",
    "get-operation",
];
const ALLOWED_COMMUNICATIONS_COMMANDS: &[&str] = &["ingest", "status"];
const ALLOWED_WHATSAPP_COMMANDS: &[&str] = &["status", "start", "stop"];

pub(super) fn execute_mail_runtime_owner_command(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ExecuteMailRuntimeOwnerCommandRequestV1,
) -> Result<OwnerResult, String> {
    let (command, exit_code, stdout, stderr) = run_allowed_runtime_command(
        store,
        sessions,
        &request.owner_session_id,
        MAIL_RUNTIME_PROCESS_ID,
        &request.command,
        &request.arg,
        ALLOWED_MAIL_COMMANDS,
    )?;
    Ok(OwnerResult::ExecuteMailRuntimeOwnerCommand(
        ExecuteMailRuntimeOwnerCommandResponseV1 {
            command,
            exit_code,
            stdout,
            stderr,
        },
    ))
}

pub(super) fn execute_communications_runtime_owner_command(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ExecuteCommunicationsRuntimeOwnerCommandRequestV1,
) -> Result<OwnerResult, String> {
    let (command, exit_code, stdout, stderr) = run_allowed_runtime_command(
        store,
        sessions,
        &request.owner_session_id,
        COMMUNICATIONS_RUNTIME_PROCESS_ID,
        &request.command,
        &request.arg,
        ALLOWED_COMMUNICATIONS_COMMANDS,
    )?;
    Ok(OwnerResult::ExecuteCommunicationsRuntimeOwnerCommand(
        ExecuteCommunicationsRuntimeOwnerCommandResponseV1 {
            command,
            exit_code,
            stdout,
            stderr,
        },
    ))
}

pub(super) fn execute_whatsapp_runtime_owner_command(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ExecuteWhatsAppRuntimeOwnerCommandRequestV1,
) -> Result<OwnerResult, String> {
    if request.command == "start" {
        sessions.authorize(store, &request.owner_session_id)?;
        if !request.arg.is_empty() {
            return Err("WhatsApp managed start does not accept runtime arguments".to_owned());
        }
        let runtime_generation = macos_managed_runtime_launch::start_with_storage_configuration(
            supervisor,
            store,
            runtime_dir,
            WHATSAPP_RUNTIME_PROCESS_ID,
        )?;
        return Ok(OwnerResult::ExecuteWhatsAppRuntimeOwnerCommand(
            ExecuteWhatsAppRuntimeOwnerCommandResponseV1 {
                command: request.command,
                exit_code: 0,
                stdout: format!("whatsapp_runtime launch_state=accepted runtime_generation={runtime_generation}"),
                stderr: String::new(),
            },
        ));
    }
    let (command, exit_code, stdout, stderr) = run_allowed_runtime_command(
        store,
        sessions,
        &request.owner_session_id,
        WHATSAPP_RUNTIME_PROCESS_ID,
        &request.command,
        &request.arg,
        ALLOWED_WHATSAPP_COMMANDS,
    )?;
    Ok(OwnerResult::ExecuteWhatsAppRuntimeOwnerCommand(
        ExecuteWhatsAppRuntimeOwnerCommandResponseV1 {
            command,
            exit_code,
            stdout,
            stderr,
        },
    ))
}

fn run_allowed_runtime_command(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    owner_session_id: &str,
    binary_name: &str,
    command: &str,
    args: &[String],
    allowed: &[&str],
) -> Result<(String, u32, String, String), String> {
    sessions.authorize(store, owner_session_id)?;
    if !allowed.contains(&command) {
        return Err("runtime command is invalid".to_owned());
    }
    let minimum_args = minimum_required_args(command);
    if args.len() < minimum_args {
        return Err("runtime command is unavailable".to_owned());
    }

    let binary = runtime_binary(binary_name)?;
    let output = Command::new(&binary)
        .arg(command)
        .args(args)
        .output()
        .map_err(|_| "runtime command is unavailable".to_owned())?;

    Ok((
        command.to_owned(),
        output
            .status
            .code()
            .unwrap_or(-1)
            .try_into()
            .unwrap_or(u32::MAX),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

fn minimum_required_args(command: &str) -> usize {
    match command {
        "status" => 0,
        "begin" => 3,
        "sync" | "complete" => 2,
        "get-connection" | "get-operation" => 1,
        "ingest" => 3,
        _ => 0,
    }
}

fn runtime_binary(process_id: &str) -> Result<String, String> {
    if let Ok(binary) = std::env::var(format!("CARGO_BIN_EXE_{}", process_id.replace('-', "_"))) {
        return Ok(binary);
    }
    let fallback = candidate_bin(process_id, std::env::current_exe().ok());
    if Path::new(&fallback).exists() {
        return Ok(fallback);
    }
    Err("runtime command is unavailable".to_owned())
}

fn candidate_bin(process_id: &str, kernel_exe: Option<std::path::PathBuf>) -> String {
    let exe = if cfg!(windows) {
        format!("{process_id}.exe")
    } else {
        process_id.to_owned()
    };
    kernel_exe
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join(&exe).to_string_lossy().into_owned())
        })
        .unwrap_or(exe)
}
