//! Vault-backed account signer resolution and JWT issuance.

mod service;

pub use service::{NatsCredentialAuthorityErrorV1, NatsJwtCredentialAuthorityV1};
