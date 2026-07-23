//! Typed private Kernel control contracts.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.gateway.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/communications_query_schema.rs"));

pub mod owner_control_client;
pub mod owner_control_proof;
pub mod validation;
