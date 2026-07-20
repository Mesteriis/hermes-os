//! Test-only Blob storage conformance package.

#[cfg(test)]
#[path = "../../../../src/platform/blob/service/src/cli/mod.rs"]
pub(crate) mod blob_service_cli;

#[cfg(test)]
mod tests;
