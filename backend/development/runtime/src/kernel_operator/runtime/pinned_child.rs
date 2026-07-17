//! Bounded development-only execution of an already verified local artifact.

use std::time::Duration;

use crate::distribution::staged_artifact as staged_native_artifact;
use crate::infrastructure::filesystem::acquire_runtime_directory_lock;
use crate::kernel_operator::artifact::verify;
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::runtime::managed::execution::{
    self as bounded_managed_child_execution, ManagedChildExecutionPolicy,
};

const MAX_RUNTIME_SECONDS: u64 = 60;

pub fn run(
    data_dir_override: Option<std::path::PathBuf>,
    registration_id: &str,
    max_runtime_seconds: u64,
) -> Result<(), String> {
    if !(1..=MAX_RUNTIME_SECONDS).contains(&max_runtime_seconds) {
        return Err(format!(
            "development managed child timeout must be between 1 and {MAX_RUNTIME_SECONDS} seconds"
        ));
    }
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let artifact = verify(&store, registration_id)?;
    let staged = staged_native_artifact::stage(
        artifact.canonical_path(),
        &runtime_dir.join("staged-development"),
        &format!(
            "artifact-{}-{}",
            artifact.binding_revision(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_nanos()
        ),
        artifact.sha256(),
    )?;
    let policy = ManagedChildExecutionPolicy::new(1, Duration::from_secs(max_runtime_seconds))?;
    let execution = bounded_managed_child_execution::run(&staged, &[], &policy);
    let cleanup = staged.remove();
    let result = match (execution, cleanup) {
        (Ok(result), Ok(())) => result,
        (Err(error), _) => return Err(error),
        (Ok(_), Err(error)) => return Err(format!("staged managed child cleanup failed: {error}")),
    };
    println!("module_registration_id={registration_id}");
    println!(
        "owner_pinned_artifact_binding_revision={}",
        artifact.binding_revision()
    );
    println!("development_managed_child_attempts={}", result.attempts());
    println!("development_managed_child_exit_code={}", result.exit_code());
    Ok(())
}
