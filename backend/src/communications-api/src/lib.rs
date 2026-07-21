//! Communications API contract for ADR-0239 read-only slice.

pub const PACKAGE: &str = "hermes-communications-api";

#[derive(Clone, Debug)]
pub struct CommunicationSummary {
    pub communication_id: String,
    pub operation_id: String,
    pub source_id: String,
    pub source_kind: String,
    pub has_body: bool,
    pub has_preview: bool,
    pub is_final_window: bool,
}

#[derive(Clone, Debug)]
pub struct GetCommunicationSummary {
    pub communication_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationsClientError {
    UnknownCommunication,
    DraftValidationFailed,
    DuplicateObservation,
}

pub type CommunicationId = String;
