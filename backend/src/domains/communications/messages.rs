mod errors;
mod ids;
mod models;
mod payload;
pub mod port;
mod projection;
mod provider_channel_store;
mod provider_observation_projection;
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
pub use provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, CommunicationSignalProjectionError,
    consume_accepted_signal_event, project_accepted_signal_if_runtime_allows,
    project_provider_observation_event, project_whatsapp_content_observed,
    project_whatsapp_delivery_state_observed, replay_accepted_signal_event,
    supports_communication_projection_signal_event,
};
pub(crate) use query_parser::parse_communication_message_search_query;
pub use search::{
    MessageSearchBoolean, MessageSearchExpression, MessageSearchField, MessageSearchPredicate,
    MessageSearchPredicateOperator, append_message_search_filter,
};
pub use states::{LocalMessageState, WorkflowState};
pub use store::MessageProjectionStore;
