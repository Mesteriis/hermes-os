//! WhatsApp-owned durable storage. Communications receives only exact envelopes.

mod durable;

pub use durable::{
    WHATSAPP_SCHEMA_V1, WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
    WhatsAppClaimedCommandV1, WhatsAppHostObservationRecordV1,
    WhatsAppProviderCommandStateV1,
};

pub const PACKAGE: &str = "hermes-whatsapp-persistence";
