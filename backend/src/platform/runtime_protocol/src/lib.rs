//! Runtime lifecycle and recovery Protobuf contract.

pub mod descriptor_validation;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.runtime.v1.rs"));
}
