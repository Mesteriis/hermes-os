//! Telegram-owned PostgreSQL persistence for operational projections and Communications outbox.

mod durable;
mod schema;

pub use durable::{
    TELEGRAM_SCHEMA_V1, TelegramDurablePersistence, TelegramDurablePersistenceError,
};
pub use schema::{TELEGRAM_STORAGE_BUNDLE_REVISION_V1, telegram_storage_bundle_v1};

pub const PACKAGE: &str = "hermes-telegram-persistence";
