//! Runtime lifecycle and recovery Protobuf contract.

pub mod validation;

#[allow(clippy::large_enum_variant)]
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.runtime.v1.rs"));
}
