pub(super) const PERSONA_IDENTITY_REVIEW_EVENT_TYPE: &str = "persona_identity.review_state_changed";
pub(super) const LEGACY_PERSON_IDENTITY_REVIEW_EVENT_TYPE: &str =
    "person_identity.review_state_changed";
pub(super) const PERSONA_IDENTITY_REVIEW_SOURCE_KIND: &str = "persona_identity_review";
pub(super) const PERSONA_IDENTITY_REVIEW_SOURCE_PROVIDER: &str = "local_api";
pub(super) const PERSONA_IDENTITY_REVIEW_PREFIX: &str = "persona_identity_review:";
pub(super) const PERSONA_IDENTITY_ID_PREFIX: &str = "identity_candidate:v1:";
pub(super) const DEFAULT_LIMIT: i64 = 50;
pub(super) const MAX_LIMIT: i64 = 100;
pub(super) const MIN_LIMIT: i64 = 1;

pub(crate) fn is_persona_identity_review_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        PERSONA_IDENTITY_REVIEW_EVENT_TYPE | LEGACY_PERSON_IDENTITY_REVIEW_EVENT_TYPE
    )
}
