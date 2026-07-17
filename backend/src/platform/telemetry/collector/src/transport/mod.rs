//! Private Unix-socket telemetry ingestion.

mod quota;
mod server;

pub use server::{serve, serve_with_control};
