//! Descriptor-authenticated inherited-FD control implementation.

mod account_jwt_update;
mod framing;
mod handshake;
mod runtime;
mod topology;
mod vault_context;
mod vault_route;

pub use runtime::{serve_inherited, serve_inherited_on_channel};
