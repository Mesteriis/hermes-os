//! Development-profile adapter for fenced external capability authorization.

use std::path::PathBuf;

use crate::infrastructure::filesystem::acquire_runtime_directory_lock;
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::modules::capability::router::{
    ExternalCapabilityRouteRequest, authorize_external_route,
};

pub fn run(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
    capability_id: &str,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 authorizes a route only; it does not expose a production Gateway or target runtime transport"
    );
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let route = authorize_external_route(
        &store,
        &ExternalCapabilityRouteRequest::new(
            registration_id,
            runtime_id,
            runtime_generation,
            capability_id,
        ),
    )?;
    println!("module_registration_id={registration_id}");
    println!("external_runtime_id={runtime_id}");
    println!("external_runtime_generation={runtime_generation}");
    println!("capability_id={capability_id}");
    println!("capability_route_grant_epoch={}", route.grant_epoch());
    Ok(())
}
