//! Test-only Scheduler protocol conformance package.

#[cfg(test)]
#[path = "../../../../src/platform/scheduler/runtime/src/cli/mod.rs"]
pub(crate) mod runtime_cli;

#[cfg(test)]
mod tests;
