#![allow(dead_code)]

// The development operator composes selected private Kernel slices directly.
// Not every helper is reachable from every operator command; semantic Clippy
// lints remain enabled for this executable and the included source.

mod cli;
mod control_store;
mod distribution;
mod identity;
mod infrastructure;
mod kernel_operator;
mod modules;
mod pairing;
mod platform;
mod recovery;
mod runtime;

use clap::Parser;

use cli::operator::Cli;

fn main() {
    if let Err(error) = cli::operator::run(Cli::parse()) {
        eprintln!("development kernel operator failed: {error}");
        std::process::exit(1);
    }
}
