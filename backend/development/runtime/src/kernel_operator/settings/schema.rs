//! Development-only file adapter for Settings Schema admission.

use std::path::{Path, PathBuf};

use crate::infrastructure::filesystem::ensure_regular_file_or_absent;
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::modules::settings::schema as settings_schema_admission;

pub fn run(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    descriptor_path: &Path,
    schema_path: &Path,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 admits local schema artifacts only; it does not enable a production settings surface"
    );
    let descriptor_bytes = read_artifact(descriptor_path, "module descriptor")?;
    let schema_bytes = read_artifact(schema_path, "settings schema")?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = crate::infrastructure::filesystem::acquire_runtime_directory_lock(&runtime_dir)?;
    let binding = settings_schema_admission::admit(
        &store,
        registration_id,
        &descriptor_bytes,
        &schema_bytes,
    )?;
    println!("module_registration_id={registration_id}");
    println!("settings_schema_major={}", binding.schema_major());
    println!("settings_schema_revision={}", binding.schema_revision());
    Ok(())
}

fn read_artifact(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err(format!("{label} must be an absolute path"));
    }
    ensure_regular_file_or_absent(path, label)?;
    std::fs::read(path).map_err(|error| error.to_string())
}
