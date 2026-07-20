//! Authenticated inherited Kernel control surface for the Blob process.

mod data;
mod framing;
mod handshake;
mod runtime;
mod vault_route;

pub(crate) use runtime::serve_inherited;
