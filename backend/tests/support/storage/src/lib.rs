//! Test-only Storage Control conformance package.

#[cfg(test)]
#[path = "../../../../src/platform/storage/runtime/src/cli/arguments.rs"]
pub(crate) mod storage_runtime_arguments;

#[cfg(test)]
pub(crate) use hermes_storage_vault as storage_runtime_vault;

#[cfg(test)]
pub(crate) use storage_runtime_vault as vault;

#[cfg(test)]
#[path = "../../../../src/platform/storage/runtime/src/control/mod.rs"]
pub(crate) mod storage_runtime_control;

#[cfg(test)]
#[path = "../../../../src/platform/storage/runtime/src/admin/mod.rs"]
pub(crate) mod admin;

#[cfg(test)]
mod tests;
