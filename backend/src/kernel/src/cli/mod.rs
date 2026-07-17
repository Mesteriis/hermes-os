//! Production Kernel command-line contract.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hermes-kernel")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) data_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Status,
    Serve,
    DeviceKeyGenerate,
    InitialOwnerEnroll {
        #[arg(long)]
        owner_id: String,
        #[arg(long)]
        device_id: String,
    },
    ServerBootstrapPairing {
        #[arg(long)]
        listen_address: SocketAddr,
        #[arg(long, default_value_t = 300)]
        ttl_seconds: u64,
    },
    ControlStore {
        #[command(subcommand)]
        operation: OfflineControlStoreCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum OfflineControlStoreCommand {
    Restore {
        #[arg(long)]
        source: PathBuf,
    },
    Reset,
}
