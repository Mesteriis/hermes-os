//! Telegram integration process root for the exact Kernel-inherited runtime contract.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use hermes_runtime_protocol::{
    v1::ManagedIntegrationRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use hermes_telegram_api::TelegramProviderKind;
use hermes_telegram_runtime::{TelegramRuntimeAdmission, bootstrap, process, settings};
use hermes_telegram_tdlib::TdJsonLibrary;
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("Telegram runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Telegram runtime settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read_contract(&paths.settings_snapshot)?)
        .map_err(|_| "Telegram runtime settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Telegram runtime settings snapshot is invalid".to_owned())?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Telegram runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "Telegram runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Telegram runtime configuration is stale".to_owned());
    }
    let settings = settings::decode(&snapshot)?;
    let library = TdJsonLibrary::load_exact(&settings.tdjson_artifact_path)
        .map_err(|_| "Telegram runtime TDLib artifact is unavailable".to_owned())?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Telegram runtime configuration is invalid".to_owned())?;
    let admission = TelegramRuntimeAdmission {
        logical_owner_id: configuration.logical_owner_id.clone(),
        configuration_instance_id: configuration.configuration_instance_id.clone(),
        module_registration_id: configuration.registration_id.clone(),
        runtime_instance_id: configuration.runtime_instance_id.clone(),
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
        vault_runtime_generation: storage.vault_runtime_generation,
        api_hash_revision: settings.api_hash_revision,
        session_encryption_key_revision: settings.session_encryption_key_revision,
    };
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|_| "Telegram runtime executor is unavailable".to_owned())?;
    let admitted = runtime
        .block_on(bootstrap::open_admitted_runtime(
            library,
            descriptor,
            schema_bytes,
            &configuration.runtime_instance_id,
            settings.api_id,
            &settings.account_id,
            TelegramProviderKind::User,
            settings.database_directory,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(|_| "Telegram runtime admission was rejected".to_owned())?;
    drop(runtime);
    process::serve_admitted_provider_loop(admitted)
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let settings_snapshot = required_path(arguments, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Telegram runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
        runtime_instance_id,
    })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Telegram runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Telegram runtime arguments are invalid".to_owned())
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Telegram runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Telegram runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Telegram runtime contract is unavailable".to_owned())
}
