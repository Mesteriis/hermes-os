mod errors;
mod ids;
mod models;
mod payload;
mod projection;
mod provider_channel_store;
mod query_parser;
mod rows;
mod search;
mod states;
mod store;
mod validation;

pub use crate::platform::communications::{
    ProviderChannelMessage, ProviderCommunicationMessagePortError, ProviderHeuristicMember,
    ProviderMessageAttachmentAnchor, ProviderMessageProjectionObservationContext,
    ProviderMessageReferenceSummary,
};
pub use errors::MessageProjectionError;
pub use models::{
    MessageSearchMatchMode, MessageSearchQuery, NewProjectedMessage, ProjectedMessage,
    ProjectedMessagePage, ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowStateCount,
};
pub use projection::{
    parse_raw_email_message_from_blob, project_parsed_raw_email_message, project_raw_email_message,
    project_raw_email_message_from_blob,
};
pub use provider_channel_store::ProviderChannelMessageStore;
pub(crate) use query_parser::parse_communication_message_search_query;
pub use search::{
    MessageSearchBoolean, MessageSearchExpression, MessageSearchField, MessageSearchPredicate,
    MessageSearchPredicateOperator, append_message_search_filter,
};
pub use states::{LocalMessageState, WorkflowState};
pub use store::MessageProjectionStore;
pub use store::MessageProjectionStore as CommunicationMessageProjectionPort;
