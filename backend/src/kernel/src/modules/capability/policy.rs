//! Hard Kernel capability policy independent of owner approval decisions.

pub fn permits_external_route(capability_id: &str) -> bool {
    !capability_id.starts_with("kernel.") && !capability_id.starts_with("owner.")
}
