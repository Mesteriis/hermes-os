//! Typed private Kernel control contracts.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.gateway.v1.rs"));
}
