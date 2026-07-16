//! Canonical durable-envelope Protobuf contract.

pub mod envelope_validation;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.events.v1.rs"));
}
