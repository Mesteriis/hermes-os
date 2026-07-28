//! Managed Communications runtime contracts.

pub mod admission;
pub mod attachment_observation_consumer;
pub mod attachment_safety;
pub mod canonical_outbox;
pub mod canonical_read_cursor;
pub mod client_port;
pub mod consumer;
pub mod content_blob_client_port;
pub mod content_ticket_client_port;
pub mod content_ticket_store;
pub mod custody_worker;
pub mod domain_outbox;
pub mod event_runtime;
pub mod query;
pub mod query_client_port;
pub mod query_port;
pub mod search_access;
pub mod search_digest;
pub mod search_job;
pub mod search_projection;
pub mod search_query;
pub mod search_worker;
