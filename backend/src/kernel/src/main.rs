use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Parser, Subcommand};
use hermes_kernel_control_store::StoreHealth;
use hermes_kernel_control_store_sqlite::SqliteControlStore;

mod control_store_lifecycle;
mod development_control_plane;
mod filesystem;
mod recovery_ipc;

use control_store_lifecycle::{
    bootstrap_control_store, installation_anchor_path, open_validated_control_store,
    read_installation_anchor, reset_untrusted_control_store,
};
use development_control_plane::{
    run_external_runtime_attestation, run_initial_owner_enrollment, run_module_approval,
    run_module_registration, run_module_status, run_module_transition,
};
use filesystem::{
    acquire_runtime_directory_lock, ensure_not_symlink, ensure_owner_private_directory,
    ensure_regular_file_or_absent, resolve_data_directory, resolve_runtime_directory,
};
use recovery_ipc::serve_recovery_socket;

#[derive(Parser)]
#[command(name = "hermes-kernel")]
struct Cli {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    development_profile: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Status,
    HoldLock,
    Serve,
    InitialOwnerEnroll {
        #[arg(long)]
        owner_id: String,
        #[arg(long)]
        device_id: String,
    },
    ModuleRegister {
        #[arg(long)]
        descriptor: PathBuf,
    },
    ModuleApprove {
        #[arg(long)]
        registration_id: String,
        #[arg(long, required = true)]
        capability: Vec<String>,
    },
    ModuleTransition {
        #[arg(long)]
        registration_id: String,
        #[arg(long, value_parser = ["suspended", "revoked"])]
        state: String,
    },
    ModuleStatus {
        #[arg(long)]
        registration_id: String,
    },
    ModuleExternalAttest {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        runtime_id: String,
        #[arg(long)]
        runtime_generation: u64,
        #[arg(long)]
        distribution_sha256: String,
    },
    ControlStore {
        #[command(subcommand)]
        operation: OfflineControlStoreCommand,
    },
}

#[derive(Subcommand)]
enum OfflineControlStoreCommand {
    Restore {
        #[arg(long)]
        source: PathBuf,
    },
    Reset,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("kernel bootstrap failed: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ControlStore { operation } => run_offline_control_store(cli.data_dir, operation),
        Command::InitialOwnerEnroll { owner_id, device_id } => {
            if !cli.development_profile {
                return Err(
                    "initial owner enrollment requires a platform adapter; development use requires --development-profile".to_owned(),
                );
            }
            run_initial_owner_enrollment(cli.data_dir, &owner_id, &device_id)
        }
        Command::ModuleRegister { descriptor } => {
            require_development_profile(cli.development_profile, "module registration")?;
            run_module_registration(cli.data_dir, &descriptor)
        }
        Command::ModuleApprove {
            registration_id,
            capability,
        } => {
            require_development_profile(cli.development_profile, "module approval")?;
            run_module_approval(cli.data_dir, &registration_id, &capability)
        }
        Command::ModuleTransition {
            registration_id,
            state,
        } => {
            require_development_profile(cli.development_profile, "module transition")?;
            run_module_transition(cli.data_dir, &registration_id, &state)
        }
        Command::ModuleStatus { registration_id } => {
            require_development_profile(cli.development_profile, "module status")?;
            run_module_status(cli.data_dir, &registration_id)
        }
        Command::ModuleExternalAttest {
            registration_id,
            runtime_id,
            runtime_generation,
            distribution_sha256,
        } => {
            require_development_profile(cli.development_profile, "external runtime attestation")?;
            run_external_runtime_attestation(
                cli.data_dir,
                &registration_id,
                &runtime_id,
                runtime_generation,
                &distribution_sha256,
            )
        }
        command => run_runtime_command(cli.data_dir, command),
    }
}

fn require_development_profile(enabled: bool, operation: &str) -> Result<(), String> {
    if enabled {
        return Ok(());
    }
    Err(format!(
        "{operation} requires --development-profile until the production control plane is implemented"
    ))
}

fn run_runtime_command(data_dir_override: Option<PathBuf>, command: Command) -> Result<(), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    prepare_runtime_directories(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");
    let store = bootstrap_control_store(&data_dir, &store_path);

    match command {
        Command::Status => {
            let control_store = match store {
                Ok(store) if store.snapshot().health() == StoreHealth::Trustworthy => "trustworthy",
                Ok(_) | Err(_) => "unavailable",
            };
            println!("state=recovery_only");
            println!("control_store={control_store}");
        }
        Command::HoldLock => {
            store?;
            println!("lock_held");
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Command::Serve => serve_recovery_socket(&runtime_dir, &store_path)?,
        Command::InitialOwnerEnroll { .. }
        | Command::ModuleRegister { .. }
        | Command::ModuleApprove { .. }
        | Command::ModuleTransition { .. }
        | Command::ModuleStatus { .. }
        | Command::ModuleExternalAttest { .. } => {
            unreachable!("development control-plane commands are dispatched before runtime bootstrap")
        }
        Command::ControlStore { .. } => {
            unreachable!("offline commands are dispatched before runtime bootstrap")
        }
    }
    Ok(())
}

fn prepare_runtime_directories(data_dir: &Path) -> Result<(), String> {
    let data_dir_existed = data_dir.exists();
    ensure_not_symlink(data_dir, "data directory")?;
    std::fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
    ensure_not_symlink(data_dir, "data directory")?;
    if data_dir_existed {
        ensure_owner_private_directory(data_dir)?;
    } else {
        std::fs::set_permissions(data_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let runtime_dir = resolve_runtime_directory(data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn run_offline_control_store(
    data_dir_override: Option<PathBuf>,
    operation: OfflineControlStoreCommand,
) -> Result<(), String> {
    let data_dir = data_dir_override
        .filter(|path| path.is_absolute())
        .ok_or_else(|| {
            "offline control-store operations require an explicit absolute --data-dir".to_owned()
        })?;
    ensure_not_symlink(&data_dir, "data directory")?;
    if !data_dir.exists() {
        return Err("offline control-store data directory does not exist".to_owned());
    }
    ensure_owner_private_directory(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");

    match operation {
        OfflineControlStoreCommand::Restore { source } => {
            if !source.is_absolute() {
                return Err("restore source must be an absolute path".to_owned());
            }
            ensure_regular_file_or_absent(&source, "restore source")?;
            if !source.exists() {
                return Err("restore source does not exist".to_owned());
            }
            if source == store_path {
                return Err("restore source must differ from the target control store".to_owned());
            }
            confirm_offline_operation("restore", &data_dir)?;
            let instance_id = read_installation_anchor(&installation_anchor_path(&data_dir))?;
            let restored = SqliteControlStore::restore_from(&source, &store_path, &instance_id)
                .map_err(|error| format!("{error:?}"))?;
            println!("control_store_generation={}", restored.generation());
        }
        OfflineControlStoreCommand::Reset => {
            confirm_offline_operation("reset", &data_dir)?;
            match open_validated_control_store(&store_path) {
                Ok(store) => {
                    let reset = store
                        .advance_recovery_fences()
                        .map_err(|error| format!("{error:?}"))?;
                    println!("reset_mode=fence_existing_instance");
                    println!("control_store_generation={}", reset.generation());
                }
                Err(_) => {
                    let reset = reset_untrusted_control_store(&data_dir, &store_path)?;
                    println!("reset_mode=new_instance");
                    println!("control_store_generation={}", reset.generation());
                }
            }
        }
    }
    Ok(())
}

fn confirm_offline_operation(operation: &str, data_dir: &Path) -> Result<(), String> {
    println!("offline_control_store_operation={operation}");
    println!("target_data_dir={}", data_dir.display());
    let expected = operation.to_ascii_uppercase();
    eprint!("Type {expected} to confirm: ");
    std::io::stderr()
        .flush()
        .map_err(|error| error.to_string())?;
    let mut confirmation = String::new();
    std::io::stdin()
        .read_line(&mut confirmation)
        .map_err(|error| error.to_string())?;
    if confirmation.trim() != expected {
        return Err("offline control-store operation was not confirmed".to_owned());
    }
    Ok(())
}
