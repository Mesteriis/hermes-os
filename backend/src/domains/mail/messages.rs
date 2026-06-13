mod errors;
mod ids;
mod models;
mod payload;
mod projection;
mod rows;
mod states;
mod store;
mod validation;

pub use errors::MessageProjectionError;
pub use models::{
    NewProjectedMessage, ProjectedMessage, ProjectedMessageSummary, WorkflowStateCount,
};
pub use projection::{
    parse_raw_email_message_from_blob, project_parsed_raw_email_message, project_raw_email_message,
    project_raw_email_message_from_blob,
};
pub use states::{LocalMessageState, WorkflowState};
pub use store::MessageProjectionStore;
