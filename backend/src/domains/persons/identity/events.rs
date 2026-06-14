use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::platform::events::NewEventEnvelope;

use super::constants::{
    PERSON_IDENTITY_REVIEW_EVENT_TYPE, PERSON_IDENTITY_REVIEW_SOURCE_KIND,
    PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER,
};
use super::errors::PersonIdentityError;
use super::models::PersonIdentityReviewState;
use super::validation::{as_object, required_payload_string};

pub(super) struct ReviewCommandEvent {
    pub(super) command_id: String,
    pub(super) identity_candidate_id: String,
    pub(super) review_state: PersonIdentityReviewState,
    pub(super) actor_id: String,
    pub(super) event_id: String,
    pub(super) occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    pub(super) fn to_event(&self) -> Result<NewEventEnvelope, PersonIdentityError> {
        Ok(NewEventEnvelope::builder(
            self.event_id.clone(),
            PERSON_IDENTITY_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": PERSON_IDENTITY_REVIEW_SOURCE_KIND,
                "provider": PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id.clone(),
            }),
            json!({
                "kind": "person_identity_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id.clone() }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "identity_candidate_id": self.identity_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
pub(super) struct ReviewEvent {
    pub(super) identity_candidate_id: String,
    pub(super) review_state: PersonIdentityReviewState,
}

impl ReviewEvent {
    pub(super) fn from_payload(payload: &Value) -> Result<Self, PersonIdentityError> {
        let payload = as_object(payload)?;
        Ok(Self {
            identity_candidate_id: required_payload_string(payload, "identity_candidate_id")?,
            review_state: PersonIdentityReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}
