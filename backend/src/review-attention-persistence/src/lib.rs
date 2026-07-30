#![forbid(unsafe_code)]

mod repository;
pub mod schema;

pub use repository::{
    ApplyReviewAttentionOperationV1, ReviewAttentionPersistenceErrorV1,
    ReviewAttentionPersistenceOutcomeV1, ReviewAttentionPersistenceV1,
};

pub const PACKAGE: &str = "hermes-review-attention-persistence";
