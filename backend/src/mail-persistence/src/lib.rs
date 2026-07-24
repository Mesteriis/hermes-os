//! Mail-owned PostgreSQL persistence for delivery state and Communications outbox.

mod durable;

pub use durable::{
    MAIL_SCHEMA_V1, MailDurablePersistence, MailDurablePersistenceError,
    MailSmtpDeliveryAttemptStateV1,
};

pub const PACKAGE: &str = "hermes-mail-persistence";
