//! Typed owner-local derived-index job records; no content or key material crosses this seam.

use hermes_communications_api::{
    CommunicationBodyBlobReferenceV1, CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationObservationIdV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobOperationV1 { Index, Remove }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsDerivedIndexJobV1 {
    pub job_id: [u8; 16],
    pub operation: CommunicationsDerivedIndexJobOperationV1,
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: Option<CommunicationConversationIdV1>,
    pub blob: Option<CommunicationBodyBlobReferenceV1>,
    pub projection_revision: u32,
    pub observed_at_unix_seconds: i64,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobErrorV1 { InvalidShape }

impl CommunicationsDerivedIndexJobV1 {
    pub fn validate(&self) -> Result<(), CommunicationsDerivedIndexJobErrorV1> {
        if self.job_id.iter().all(|byte| *byte == 0) || self.projection_revision == 0 {
            return Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape);
        }
        match self.operation {
            CommunicationsDerivedIndexJobOperationV1::Index if self.conversation_id.is_some() && self.blob.is_some() => Ok(()),
            CommunicationsDerivedIndexJobOperationV1::Remove if self.conversation_id.is_none() && self.blob.is_none() => Ok(()),
            _ => Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape),
        }
    }
}
