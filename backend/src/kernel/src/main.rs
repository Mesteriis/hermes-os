use clap::Parser;

mod cli;
mod control_store;
mod distribution;
mod identity;
mod infrastructure;
mod modules;
mod platform;
mod recovery;
mod runtime;

use cli::{Cli, Command};

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("kernel bootstrap failed: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ControlStore { operation } => control_store::offline::run(cli.data_dir, operation),
        Command::DeviceKeyGenerate => identity::device::generation::run(cli.data_dir),
        Command::InitialOwnerEnroll {
            owner_id,
            device_id,
        } => identity::enrollment::initial::run(cli.data_dir, &owner_id, &device_id),
        Command::ServerBootstrapPairing {
            listen_address,
            ttl_seconds,
        } => identity::server_pairing::bootstrap::run(cli.data_dir, listen_address, ttl_seconds),
        command => runtime::run(cli.data_dir, command),
    }
}
