#![forbid(unsafe_code)]

mod model;
mod outbox;
mod repository;
pub mod schema;

pub use model::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, ReviewedTaskCandidatePromotionOutcomeV1,
    UnpublishedPromotionEventV1,
};
pub use repository::ReviewedTaskCandidatePromotionPersistenceV1;

pub const PACKAGE: &str = "hermes-reviewed-task-candidate-promotion-persistence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedTaskCandidatePromotionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    ApprovalConflict,
    ResultConflict,
    OutboxConflict,
    NotFound,
}
