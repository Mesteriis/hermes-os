//! Storage-side authenticated inherited Kernel control transport.

mod apply;
mod framing;
mod handshake;
mod revocation;
mod runtime;
#[allow(dead_code)]
mod vault_route;

#[allow(unused_imports)]
pub use handshake::describe;
#[allow(unused_imports)]
pub use handshake::describe_on_channel;
#[allow(unused_imports)]
pub use runtime::{
    serve_credential_bootstrapped_on_channel, serve_inherited, serve_inherited_on_channel,
    serve_on_channel,
};
#[allow(unused_imports)]
pub use vault_route::InheritedVaultRoutePortV1;
