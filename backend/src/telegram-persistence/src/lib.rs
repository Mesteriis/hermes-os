//! Telegram-owned PostgreSQL persistence for operational projections and Communications outbox.

mod durable;

pub use durable::{
    TELEGRAM_SCHEMA_V1, TelegramDurablePersistence, TelegramDurablePersistenceError,
};

pub const PACKAGE: &str = "hermes-telegram-persistence";
