//! Development-only adapter for persisted Settings lifecycle transitions.

use std::path::PathBuf;

use crate::infrastructure::filesystem::acquire_runtime_directory_lock;
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::modules::settings::application::{
    self as settings_apply_lifecycle, parse_acknowledgement,
};

pub fn run(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    revision: u64,
    acknowledgement: &str,
    reason_code: Option<&str>,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 records a lifecycle acknowledgement; it does not contact a module runtime"
    );
    let acknowledgement = parse_acknowledgement(acknowledgement, reason_code)?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    settings_apply_lifecycle::acknowledge(&store, registration_id, revision, acknowledgement)?;
    println!("module_registration_id={registration_id}");
    println!("settings_revision={revision}");
    Ok(())
}
