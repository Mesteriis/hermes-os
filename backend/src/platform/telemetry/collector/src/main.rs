//! Telemetry Collector process entrypoint.

mod cli;
mod control;
mod storage;
mod transport;

fn main() -> Result<(), String> {
    cli::run()
}
