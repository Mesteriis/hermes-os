//! Current managed-module runtime authority derived from durable Kernel state.

use hermes_kernel_control_store::RuntimeTrustStore;

pub(crate) fn current_managed_runtime_matches<S>(
    store: &S,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
) -> Result<bool, S::Error>
where
    S: RuntimeTrustStore,
{
    let Some(binding) = store.effective_bundled_managed_launch_binding(registration_id)? else {
        return Ok(false);
    };
    let Some(launch) = store.effective_managed_launch_record(registration_id)? else {
        return Ok(false);
    };
    Ok(launch.binding_revision() == binding.binding_revision()
        && launch.runtime_instance_id() == runtime_instance_id
        && launch.runtime_generation() == runtime_generation
        && launch.grant_epoch() == grant_epoch)
}
