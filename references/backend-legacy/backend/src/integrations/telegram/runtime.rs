mod actor;
mod commands;
pub(crate) mod manager;
pub(crate) mod models;
mod participant_commands;
mod state;
mod status;
#[cfg(test)]
mod tests;
mod validation;

const TDJSON_BOOTSTRAP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_COMMAND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_RECEIVE_POLL_SECONDS: f64 = 1.0;
