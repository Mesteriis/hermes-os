//! Telegram-owned PostgreSQL persistence for operational projections and Communications outbox.

mod communications_outbox;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod durable;
mod schema;

pub use communications_outbox::TelegramCommunicationsOutboxStoreV1;
#[cfg(feature = "conformance-test-support")]
pub use conformance::TelegramPersistenceConformanceV1;
pub use durable::{
    TELEGRAM_SCHEMA_V1, TelegramDurablePersistence, TelegramDurablePersistenceError,
};
pub use schema::{TELEGRAM_STORAGE_BUNDLE_REVISION_V1, telegram_storage_bundle_v1};

pub const PACKAGE: &str = "hermes-telegram-persistence";
