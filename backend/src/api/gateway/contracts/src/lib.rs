//! Typed recovery-only Core Gateway contract.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.gateway.v1.rs"));
}
