//! Durable technical accounting for Blob bytes and fenced deletion reservations.

mod codec;
mod record;
mod store;

pub use record::{BlobDeletionReservationV1, BlobWriteReservationV1};
pub use store::{BlobMetadataError, BlobMetadataLedger};
