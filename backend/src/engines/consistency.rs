mod constants;
mod engine;
mod errors;
pub(crate) mod evidence;
mod helpers;
mod models;
mod parsing;
mod rows;
mod store;
mod validation;

pub use engine::ConsistencyEngine;
pub use errors::ConsistencyError;
pub use helpers::contradiction_observation_id;
pub use models::{
    AcceptedClaim, ContradictionObservation, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind, EvidenceClaimExtractionInput, NewContradictionObservation,
    NewEvidenceClaim,
};
pub use store::ContradictionObservationStore;
pub use store::ContradictionObservationStore as ContradictionObservationPort;
