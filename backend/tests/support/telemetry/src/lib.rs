//! Test-only telemetry contract conformance package.

#[cfg(test)]
mod collector;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use collector::storage;
