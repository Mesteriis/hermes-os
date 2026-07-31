#![forbid(unsafe_code)]

mod admission;
mod contracts;

pub use admission::{
    REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1, review_task_candidate_module_descriptor_v1,
    review_task_candidate_settings_schema_bytes_v1, review_task_candidate_settings_schema_v1,
};

pub const PACKAGE: &str = "hermes-review-task-candidate-runtime";
