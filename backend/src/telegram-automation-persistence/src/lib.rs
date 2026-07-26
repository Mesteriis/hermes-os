//! Telegram-owned persistence for automation templates, policies and preview receipts.

mod repository;
pub mod schema;

pub use repository::{
    PersistedMutation, TelegramAutomationPersistence, TelegramAutomationPersistenceError,
};

pub const PACKAGE: &str = "hermes-telegram-automation-persistence";
