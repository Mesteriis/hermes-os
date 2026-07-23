//! Communications-owned attachment safety transition use case.

use hermes_communications_api::{
    AttachmentSafetyTransitionCommandV1, AttachmentSafetyTransitionDecisionV1,
};
use hermes_communications_domain::decide_attachment_safety_transition;
use hermes_communications_persistence::CommunicationsDurablePersistence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyTransitionApplyErrorV1 {
    InvalidTransition,
    Conflict,
    Unavailable,
}

pub async fn apply_attachment_safety_transition(
    persistence: &CommunicationsDurablePersistence,
    command: AttachmentSafetyTransitionCommandV1,
) -> Result<AttachmentSafetyTransitionDecisionV1, AttachmentSafetyTransitionApplyErrorV1> {
    let decision = decide_attachment_safety_transition(command)
        .map_err(|_| AttachmentSafetyTransitionApplyErrorV1::InvalidTransition)?;
    let applied = persistence
        .compare_and_set_attachment_safety_state(
            decision.attachment_anchor_id,
            decision.expected_state,
            decision.next_state,
            decision.evidence_id,
            decision.observed_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSafetyTransitionApplyErrorV1::Unavailable)?;
    applied
        .then_some(decision)
        .ok_or(AttachmentSafetyTransitionApplyErrorV1::Conflict)
}
