//! Blob service composition root for the inherited managed-child contract.

mod cli;
mod control;

use hermes_runtime_protocol::{
    v1::BlobRuntimeConfigurationV1, validation::blob::validate_blob_runtime_configuration,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == "serve-inherited" => serve_inherited(&mut arguments),
        _ => Err("Blob service command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let paths = cli::parse_serve_inherited_arguments(arguments)?;
    control::serve_inherited(
        read_contract_file(&paths.descriptor_path)?,
        paths
            .settings_schema_path
            .map_or_else(|| Ok(Vec::new()), |path| read_contract_file(&path))?,
        read_configuration(&paths.configuration_path)?,
    )
}

fn read_configuration(path: &std::path::Path) -> Result<BlobRuntimeConfigurationV1, String> {
    let bytes = read_contract_file(path)?;
    let configuration = BlobRuntimeConfigurationV1::decode(bytes.as_slice())
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    validate_blob_runtime_configuration(&configuration)
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &std::path::Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Blob runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Blob runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Blob runtime contract is unavailable".to_owned())
}
