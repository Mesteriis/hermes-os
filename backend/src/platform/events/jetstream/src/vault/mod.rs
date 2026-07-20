//! Ciphertext-only Vault route for per-runtime NATS credentials.

mod account_signer;
mod context;
mod credentials;
mod event_hub;
mod port;
mod resolver_credentials;
mod session;

pub use account_signer::{
    NatsAccountSignerFenceV1, NatsAccountSignerLeaseAdapterV1, NatsAccountSignerLeaseErrorV1,
};
pub use context::NatsVaultRouteContextV1;
pub use credentials::{
    NatsCredentialLeaseAdapterV1, NatsCredentialLeaseErrorV1, NatsRuntimeCredentialFenceV1,
};
pub use event_hub::{
    EventHubCredentialFenceV1, EventHubCredentialLeaseAdapterV1, EventHubCredentialLeaseErrorV1,
};
pub use port::{NatsVaultRouteFailureV1, NatsVaultRoutePortV1};
pub use resolver_credentials::{
    NatsResolverCredentialFenceV1, NatsResolverCredentialLeaseAdapterV1,
    NatsResolverCredentialLeaseErrorV1,
};
