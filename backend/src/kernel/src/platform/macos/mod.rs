//! macOS release binding, signature verification, and native launches.

pub(crate) mod bundled_release;
pub(crate) mod host_bridge_descriptor;
#[allow(dead_code)]
pub(crate) mod code_signature;
pub(crate) mod managed_launch;
#[allow(dead_code)]
pub(crate) mod native_launch;
#[allow(dead_code)]
pub(crate) mod release_resources;
