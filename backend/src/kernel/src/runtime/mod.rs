//! Online Kernel runtime dispatch.

pub(crate) mod external;
pub(crate) mod lifecycle;
pub(crate) mod managed;

use std::path::PathBuf;

use hermes_kernel_control_store::StoreHealth;

use crate::cli::{Command, DeveloperModeCommand};
use crate::control_store::lifecycle::bootstrap_control_store;
use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, resolve_data_directory, resolve_runtime_directory,
};
use crate::infrastructure::paths::prepare_runtime_directories;
use crate::platform::control_plane::serve as serve_platform_control_plane;
use crate::recovery::serve_recovery_socket;
use crate::runtime::lifecycle::shutdown::install as install_shutdown_signal;

pub(crate) fn run(data_dir_override: Option<PathBuf>, command: Command) -> Result<(), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir = prepare_runtime_directories(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");
    match command {
        Command::Status => print_status(bootstrap_control_store(&data_dir, &store_path)),
        Command::Serve { browser_gateway } => {
            let store = bootstrap_control_store(&data_dir, &store_path);
            let developer_mode_enabled = store
                .as_ref()
                .map_err(Clone::clone)?
                .developer_mode_enabled()
                .map_err(|error| format!("{error:?}"))?;
            let browser_gateway = browser_gateway.into_configuration(developer_mode_enabled)?;
            serve(store, &data_dir, &runtime_dir, &store_path, browser_gateway)
        }
        Command::DeveloperMode { operation } => {
            configure_developer_mode(bootstrap_control_store(&data_dir, &store_path)?, operation)
        }
        _ => unreachable!("non-runtime command was dispatched to runtime"),
    }
}

fn configure_developer_mode(
    store: hermes_kernel_control_store_sqlite::SqliteControlStore,
    operation: DeveloperModeCommand,
) -> Result<(), String> {
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("developer mode requires an enrolled initial owner".to_owned());
    }
    match operation {
        DeveloperModeCommand::Status => {}
        DeveloperModeCommand::Enable => store
            .set_developer_mode_enabled(true)
            .map_err(|error| format!("{error:?}"))?,
        DeveloperModeCommand::Disable => store
            .set_developer_mode_enabled(false)
            .map_err(|error| format!("{error:?}"))?,
    }
    let enabled = store
        .developer_mode_enabled()
        .map_err(|error| format!("{error:?}"))?;
    println!(
        "developer_mode={}",
        if enabled { "enabled" } else { "disabled" }
    );
    println!("developer_mode_ingress=private_lan_http_only");
    println!("developer_mode_egress=unrestricted");
    Ok(())
}

fn print_status(
    store: Result<hermes_kernel_control_store_sqlite::SqliteControlStore, String>,
) -> Result<(), String> {
    let (state, control_store) = match store {
        Ok(store) if store.snapshot().health() == StoreHealth::Trustworthy => {
            ("module_control_plane", "trustworthy")
        }
        Ok(_) | Err(_) => ("recovery_only", "unavailable"),
    };
    println!("state={state}");
    println!("control_store={control_store}");
    Ok(())
}

fn serve(
    store: Result<hermes_kernel_control_store_sqlite::SqliteControlStore, String>,
    data_dir: &std::path::Path,
    runtime_dir: &std::path::Path,
    store_path: &std::path::Path,
    browser_gateway: Option<crate::platform::gateway::BrowserGatewayConfigurationV1>,
) -> Result<(), String> {
    match store {
        Ok(store) if store.snapshot().health() == StoreHealth::Trustworthy => {
            serve_platform_control_plane(store, data_dir, runtime_dir, store_path, browser_gateway)
        }
        Ok(_) | Err(_) if browser_gateway.is_none() => {
            serve_recovery_socket(runtime_dir, store_path, None, install_shutdown_signal()?)
        }
        Ok(_) | Err(_) => Err("browser Gateway requires a trustworthy control store".to_owned()),
    }
}
