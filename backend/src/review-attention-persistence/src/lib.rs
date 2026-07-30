#![forbid(unsafe_code)]

mod query;
mod realtime;
mod repository;
pub mod schema;

pub use query::{ReviewAttentionListFilterV1, ReviewAttentionPageV1};
pub use realtime::ReviewAttentionRealtimeTransitionV1;
pub use repository::{
    ApplyReviewAttentionOperationV1, ReviewAttentionPersistenceErrorV1,
    ReviewAttentionPersistenceOutcomeV1, ReviewAttentionPersistenceV1,
};

pub const PACKAGE: &str = "hermes-review-attention-persistence";
