//! Exact-byte durable outbox record accepted before broker publication.

use sha2::{Digest, Sha256};

use crate::validation::envelope::decode_envelope_v1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboxRecordError {
    InvalidEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRecordV1 {
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    exact_bytes: Vec<u8>,
}

impl OutboxRecordV1 {
    pub fn accept(exact_bytes: Vec<u8>) -> Result<Self, OutboxRecordError> {
        let envelope =
            decode_envelope_v1(&exact_bytes).map_err(|_| OutboxRecordError::InvalidEnvelope)?;
        let message_id = envelope
            .message_id
            .as_slice()
            .try_into()
            .map_err(|_| OutboxRecordError::InvalidEnvelope)?;
        Ok(Self {
            message_id,
            envelope_sha256: Sha256::digest(&exact_bytes).into(),
            exact_bytes,
        })
    }

    #[must_use]
    pub const fn message_id(&self) -> &[u8; 16] {
        &self.message_id
    }

    #[must_use]
    pub const fn envelope_sha256(&self) -> &[u8; 32] {
        &self.envelope_sha256
    }

    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.exact_bytes
    }
}
