//! Collector command parsing and composition.

use std::path::PathBuf;

use crate::storage::{TelemetryRetentionV1, TelemetrySegmentStore};
use crate::transport;

pub fn run() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments
        .next()
        .ok_or_else(|| "Telemetry Collector command is required".to_owned())?;
    match command.to_str() {
        Some("serve") => serve(&mut arguments),
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Telemetry Collector command is unavailable".to_owned()),
    }
}

fn serve(arguments: &mut std::env::ArgsOs) -> Result<(), String> {
    let data_dir = required_path(arguments, "--data-dir")?;
    let runtime_dir = required_path(arguments, "--runtime-dir")?;
    no_remaining_arguments(arguments)?;
    let store = TelemetrySegmentStore::open(data_dir, TelemetryRetentionV1::production_default())?;
    transport::serve(&runtime_dir, store)
}

fn serve_inherited(arguments: &mut std::env::ArgsOs) -> Result<(), String> {
    let data_dir = required_path(arguments, "--data-dir")?;
    let runtime_dir = required_path(arguments, "--runtime-dir")?;
    let descriptor_path = required_path(arguments, "--descriptor-path")?;
    let schema_path = required_path(arguments, "--settings-schema-path")?;
    no_remaining_arguments(arguments)?;
    let descriptor = std::fs::read(descriptor_path)
        .map_err(|_| "Telemetry Collector contract is unavailable".to_owned())?;
    let schema = std::fs::read(schema_path)
        .map_err(|_| "Telemetry Collector contract is unavailable".to_owned())?;
    let control = crate::control::describe(descriptor, schema)?;
    let store = TelemetrySegmentStore::open(data_dir, TelemetryRetentionV1::production_default())?;
    transport::serve_with_control(&runtime_dir, store, control)
}

fn required_path(arguments: &mut std::env::ArgsOs, name: &str) -> Result<PathBuf, String> {
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new(name)) {
        return Err("Telemetry Collector arguments are invalid".to_owned());
    }
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Telemetry Collector arguments are invalid".to_owned())
}

fn no_remaining_arguments(arguments: &mut std::env::ArgsOs) -> Result<(), String> {
    if arguments.next().is_some() {
        return Err("Telemetry Collector arguments are invalid".to_owned());
    }
    Ok(())
}
