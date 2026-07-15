use super::*;

pub(super) fn identity_candidate_observation(
    payload: &PersonaIdentityCandidatePayload,
) -> NewObservation {
    let identity_candidate_id = payload.identity_candidate_id();
    NewObservation::new(
        "PERSONA_IDENTITY_CANDIDATE",
        ObservationOriginKind::LocalRuntime,
        chrono::Utc::now(),
        json!({
            "identity_candidate_id": identity_candidate_id,
            "candidate_kind": payload.candidate_kind.as_str(),
            "left_persona_id": payload.left_persona_id,
            "right_persona_id": payload.right_persona_id,
            "email_address": payload.email_address,
            "evidence_summary": payload.evidence_summary,
            "confidence": payload.confidence,
        }),
        format!("identity-candidate://{identity_candidate_id}"),
    )
    .confidence(payload.confidence)
    .provenance(json!({
        "pipeline": "persona_identity_candidates",
        "candidate_kind": payload.candidate_kind.as_str(),
    }))
}

pub(super) fn identity_candidate_review_item(
    payload: &PersonaIdentityCandidatePayload,
) -> NewReviewItem {
    NewReviewItem::new(
        ReviewItemKind::IdentityCandidate,
        payload.candidate_kind.as_str(),
        payload.evidence_summary.clone(),
        payload.confidence,
    )
    .metadata(json!({
        "mirrored_from": "identity_candidates",
        "identity_candidate_id": payload.identity_candidate_id(),
        "candidate_kind": payload.candidate_kind.as_str(),
        "left_persona_id": payload.left_persona_id,
        "right_persona_id": payload.right_persona_id,
        "email_address": payload.email_address,
    }))
}

pub(super) fn identity_candidate_review_evidence(
    identity_candidate_id: &str,
    observation_id: &str,
) -> NewReviewItemEvidence {
    NewReviewItemEvidence::new(observation_id.to_owned())
        .role("primary")
        .metadata(json!({
            "mirrored_from": "identity_candidates",
            "identity_candidate_id": identity_candidate_id,
        }))
}
