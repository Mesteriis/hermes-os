//! WhatsApp integration process composition root.

use std::fs;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use hermes_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1;
use hermes_whatsapp_api::{WhatsAppProviderCommand, WhatsAppProviderEvent};
use hermes_whatsapp_core::WhatsAppTransportError;
use hermes_whatsapp_persistence::WhatsAppDurablePersistence;
use hermes_whatsapp_runtime::{
    WhatsAppProviderTransport, WhatsAppRuntime, WhatsAppTransportResponse,
    bootstrap::open_admitted_runtime,
    process::WhatsAppProcessLoop,
};
use prost::Message;

struct HostBridgeTransport;

impl WhatsAppProviderTransport for HostBridgeTransport {
    fn execute(
        &mut self,
        _command: &WhatsAppProviderCommand,
    ) -> Result<WhatsAppTransportResponse, WhatsAppTransportError> {
        Err(WhatsAppTransportError::Rejected)
    }

    fn poll_events(&mut self) -> Result<Vec<WhatsAppProviderEvent>, WhatsAppTransportError> {
        Ok(Vec::new())
    }
}

async fn serve(
    socket_path: &Path,
    durable: &WhatsAppDurablePersistence,
    handle: &tokio::runtime::Handle,
) -> Result<(), String> {
    if !socket_path.is_absolute() {
        return Err("WhatsApp runtime socket path must be absolute".to_owned());
    }
    if socket_path.exists() {
        fs::remove_file(socket_path)
            .map_err(|error| format!("failed to replace WhatsApp runtime socket: {error}"))?;
    }
    let listener = UnixListener::bind(socket_path)
        .map_err(|error| format!("failed to bind WhatsApp runtime socket: {error}"))?;
    let mut process = WhatsAppProcessLoop::new(WhatsAppRuntime::new(HostBridgeTransport));
    for stream in listener.incoming() {
        let stream = stream.map_err(|error| format!("WhatsApp runtime accept failed: {error}"))?;
        process
            .serve_client_connection_durable(stream, durable, handle)
            .map_err(|error| format!("WhatsApp runtime client failed: {error:?}"))?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "serve-inherited" => serve_inherited(std::env::args_os().skip(2).peekable()),
        "status" => {
            println!(
                "whatsapp_runtime state=blocked host_transport=admitted credential_handoff=required"
            );
            Ok(())
        }
        "start" => Err(
            "WhatsApp runtime requires Kernel-managed admission with a scoped Vault credential lease; direct database URL environment is forbidden"
                .to_owned(),
        ),
        "stop" => Err("WhatsApp runtime stop requires the Kernel supervisor".to_owned()),
        other => Err(format!("WhatsApp runtime command is unavailable: {other}")),
    }
}

fn serve_inherited<I>(mut arguments: std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let descriptor_path = required_path(&mut arguments, "--descriptor-path")?;
    let settings_schema_path = required_path(&mut arguments, "--settings-schema-path")?;
    let configuration_path = required_path(&mut arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_path(&mut arguments, "--runtime-instance-id")?
        .to_string_lossy()
        .into_owned();
    if arguments.next().is_some() {
        return Err("WhatsApp managed runtime arguments are invalid".to_owned());
    }
    let socket = std::env::var_os("HERMES_WHATSAPP_RUNTIME_SOCKET")
        .ok_or_else(|| "HERMES_WHATSAPP_RUNTIME_SOCKET is required".to_owned())?;
    let configuration = read_configuration(&configuration_path)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .map_err(|error| format!("failed to build WhatsApp runtime executor: {error}"))?;
    runtime.block_on(async {
        let admitted = open_admitted_runtime(
            read_contract_file(&descriptor_path)?,
            read_contract_file(&settings_schema_path)?,
            &runtime_instance_id,
            configuration,
        )
        .await
        .map_err(|_| "WhatsApp managed runtime admission failed".to_owned())?;
        let handle = tokio::runtime::Handle::current();
        serve(Path::new(&socket), &admitted.durable, &handle).await
    })
}

fn required_path<I>(arguments: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new(name)) {
        return Err("WhatsApp managed runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "WhatsApp managed runtime arguments are invalid".to_owned())
}

fn read_configuration(path: &Path) -> Result<ManagedStorageRuntimeConfigurationV1, String> {
    let configuration = ManagedStorageRuntimeConfigurationV1::decode(
        read_contract_file(path)?.as_slice(),
    )
    .map_err(|_| "WhatsApp managed runtime configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "WhatsApp managed runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("WhatsApp managed runtime contract is unavailable".to_owned());
    }
    fs::read(path).map_err(|_| "WhatsApp managed runtime contract is unavailable".to_owned())
}
