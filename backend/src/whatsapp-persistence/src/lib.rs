//! WhatsApp-owned durable storage. Communications receives only exact envelopes.

mod durable;

pub use durable::{
    WHATSAPP_SCHEMA_V1, WhatsAppClaimedCommandV1, WhatsAppDurablePersistence,
    WhatsAppDurablePersistenceError, WhatsAppHostObservationRecordV1,
    WhatsAppProviderCommandCompletionV1, WhatsAppProviderCommandStateV1,
};

pub const PACKAGE: &str = "hermes-whatsapp-persistence";
