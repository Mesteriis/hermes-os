//! Kernel-inherited process root for the canonical Communications domain.

use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_communications_runtime::{
    admission::{
        communications_module_descriptor_v1, communications_settings_schema_bytes_v1,
    },
    event_runtime::{CommunicationsEventRuntimeV1, CommunicationsRuntimeAdmissionV1},
};
use hermes_communications_persistence::communications_storage_bundle_v1;
use hermes_runtime_protocol::{
    v1::ManagedDomainRuntimeConfigurationV1,
    validation::{
        descriptor::decode_settings_schema_v1,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => serve_inherited(&mut arguments),
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            export_storage_bundle(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            export_module_descriptor(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_settings_schema(&mut arguments)
        }
        _ => Err("Communications runtime command is unavailable".to_owned()),
    }
}

fn export_storage_bundle<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_storage_bundle_v1().encode_to_vec(),
    )
    .map_err(|_| "Communications storage bundle is unavailable".to_owned())
}

fn export_module_descriptor<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let build_id = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Communications descriptor build id is required".to_owned())?;
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_module_descriptor_v1(&build_id).encode_to_vec(),
    )
    .map_err(|_| "Communications module descriptor is unavailable".to_owned())
}

fn export_settings_schema<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_settings_schema_bytes_v1(),
    )
    .map_err(|_| "Communications settings schema is unavailable".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Communications runtime settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Communications runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Communications runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Communications runtime configuration is stale".to_owned());
    }
    let storage = configuration.storage.clone()
        .ok_or_else(|| "Communications runtime configuration is invalid".to_owned())?;
    let admission = CommunicationsRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Communications runtime executor is unavailable".to_owned())?;
    let mut control_channel = inherited_control_channel()?;
    let mut runtime = executor.block_on(CommunicationsEventRuntimeV1::open(
        &mut control_channel,
        descriptor,
        schema_bytes,
        &admission,
        &configuration.event_hub_endpoint,
        configuration.event_credential_revision,
        storage,
    )).map_err(|_| "Communications runtime admission was rejected".to_owned())?;
    loop {
        executor.block_on(consume_or_tick(&mut runtime))?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_| "Communications runtime clock is unavailable".to_owned())?;
        let now = i64::try_from(now.as_secs())
            .map_err(|_| "Communications runtime clock is unavailable".to_owned())?;
        executor.block_on(runtime.relay_domain_outbox(now))
            .map_err(|_| "Communications runtime outbox relay failed".to_owned())?;
    }
}

async fn consume_or_tick(runtime: &mut CommunicationsEventRuntimeV1) -> Result<(), String> {
    if runtime
        .try_handle_client_delivery()
        .await
        .map_err(|_| "Communications runtime client delivery failed".to_owned())?
    {
        return Ok(());
    }
    tokio::select! {
        result = runtime.consume_next() => result
            .map_err(|_| "Communications runtime event delivery failed".to_owned()),
        () = tokio::time::sleep(Duration::from_secs(1)) => runtime
            .process_next_derived_index_job()
            .await
            .map(|_| ())
            .map_err(|_| "Communications runtime derived index worker failed".to_owned()),
    }
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Communications runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths { descriptor, settings_schema, runtime_configuration, runtime_instance_id })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where I: Iterator<Item = OsString>, { required_string(arguments, name).map(PathBuf::from) }

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where I: Iterator<Item = OsString>, {
    if arguments.next().as_deref() != Some(OsStr::new(name)) { return Err("Communications runtime arguments are invalid".to_owned()); }
    arguments.next().and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Communications runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 { return Err("Communications runtime inherited control channel is unavailable".to_owned()); }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Communications runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_CONTRACT_BYTES {
        return Err("Communications runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Communications runtime contract is unavailable".to_owned())
}
