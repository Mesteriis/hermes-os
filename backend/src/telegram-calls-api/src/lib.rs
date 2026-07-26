pub mod contract;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.telegram.calls.v1.rs"));
}

pub use contract::*;

pub const PACKAGE: &str = "hermes-telegram-calls-api";
