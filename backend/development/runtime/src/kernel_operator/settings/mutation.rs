//! Development-only file adapter for one operator Settings revision.

use std::path::{Path, PathBuf};

use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, ensure_regular_file_or_absent, resolve_data_directory,
};
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::modules::settings::mutation as settings_operator_mutation;

pub fn run(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    expected_revision: u64,
    snapshot_path: &Path,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 commits a local settings revision only; it does not apply runtime configuration"
    );
    if !snapshot_path.is_absolute() {
        return Err("settings snapshot must be an absolute path".to_owned());
    }
    ensure_regular_file_or_absent(snapshot_path, "settings snapshot")?;
    let snapshot_bytes = std::fs::read(snapshot_path).map_err(|error| error.to_string())?;
    let data_dir = resolve_data_directory(data_dir_override.clone())?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let revision = settings_operator_mutation::commit(
        &data_dir,
        &store,
        registration_id,
        expected_revision,
        &snapshot_bytes,
    )?;
    println!("module_registration_id={registration_id}");
    println!("settings_desired_revision={revision}");
    println!("settings_apply_state=pending_validation");
    Ok(())
}
