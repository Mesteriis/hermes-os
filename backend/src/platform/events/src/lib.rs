//! Canonical durable-envelope Protobuf contract.

pub mod validation;

pub use validation::envelope;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.events.v1.rs"));
}
