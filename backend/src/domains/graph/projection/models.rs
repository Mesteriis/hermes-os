use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domains::graph::core::RelationshipType;

/// Counts deterministic projection operations attempted during a V1 graph projection run.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct GraphProjectionReport {
    pub nodes_upserted: usize,
    pub edges_upserted: usize,
    pub evidence_upserted: usize,
}

pub(super) struct PersonRow {
    pub(super) person_id: String,
    pub(super) display_name: String,
    pub(super) email_address: String,
}

pub(super) struct MessageRow {
    pub(super) message_id: String,
    pub(super) raw_record_id: String,
    pub(super) account_id: String,
    pub(super) provider_record_id: String,
    pub(super) subject: String,
    pub(super) sender: String,
    pub(super) recipients: Vec<String>,
    pub(super) body_text: String,
    pub(super) occurred_at: Option<DateTime<Utc>>,
}

pub(super) struct DocumentRow {
    pub(super) document_id: String,
    pub(super) document_kind: String,
    pub(super) title: String,
    pub(super) source_fingerprint: String,
    pub(super) imported_at: DateTime<Utc>,
}

pub(super) enum MessageEndpoint {
    Person { node_id: String },
    EmailAddress { node_id: String },
}

impl MessageEndpoint {
    pub(super) fn node_id(&self) -> &str {
        match self {
            Self::Person { node_id } | Self::EmailAddress { node_id } => node_id,
        }
    }

    pub(super) fn relationship_type(&self, direction: RelationshipDirection) -> RelationshipType {
        match (self, direction) {
            (Self::Person { .. }, RelationshipDirection::Sent) => {
                RelationshipType::PersonSentMessage
            }
            (Self::Person { .. }, RelationshipDirection::Received) => {
                RelationshipType::PersonReceivedMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Sent) => {
                RelationshipType::EmailAddressSentMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Received) => {
                RelationshipType::EmailAddressReceivedMessage
            }
        }
    }

    pub(super) fn project_relationship_type(&self) -> RelationshipType {
        match self {
            Self::Person { .. } => RelationshipType::ProjectInvolvesPerson,
            Self::EmailAddress { .. } => RelationshipType::ProjectInvolvesEmailAddress,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum RelationshipDirection {
    Sent,
    Received,
}
