//! WhatsApp-owned durable storage. Communications receives only exact envelopes.

mod durable;
mod schema;

pub use durable::{
    WhatsAppClaimedCommandV1, WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
    WhatsAppHostObservationRecordV1, WhatsAppProviderCommandEnqueueV1,
    WhatsAppProviderCommandStateV1, WhatsAppProviderCommandStatusV1,
};
pub use schema::{
    WHATSAPP_SCHEMA_V1, WHATSAPP_STORAGE_BUNDLE_REVISION_V1, whatsapp_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-whatsapp-persistence";
