//! Generated public contract for Telegram-owned automation management and preview.

pub mod contract;
pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.telegram.automation.v1.rs"
    ));
}

pub const PACKAGE: &str = "hermes-telegram-automation-api";
