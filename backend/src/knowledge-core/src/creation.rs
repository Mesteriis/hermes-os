use crate::model::validate_draft;
use crate::{
    KnowledgeValidationErrorV1, ReviewedCandidateKnowledgeNoteDraftV1,
    VerifiedKnowledgeNoteStatusV1, VerifiedKnowledgeNoteV1, derive_verified_knowledge_note_id_v1,
    validate_verified_knowledge_note_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeNoteCreationErrorV1 {
    InvalidDraft,
}

pub fn create_verified_knowledge_note_from_reviewed_candidate_v1(
    draft: ReviewedCandidateKnowledgeNoteDraftV1,
) -> Result<VerifiedKnowledgeNoteV1, KnowledgeNoteCreationErrorV1> {
    validate_draft(&draft).map_err(invalid_draft)?;
    let note_id = derive_verified_knowledge_note_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )
    .map_err(invalid_draft)?;
    let note = VerifiedKnowledgeNoteV1 {
        note_id,
        logical_owner_id: draft.logical_owner_id,
        title: draft.title,
        excerpt: draft.excerpt,
        topic_hints: draft.topic_hints,
        source_basis: draft.source_basis,
        confidence_basis_points: draft.confidence_basis_points,
        status: VerifiedKnowledgeNoteStatusV1::Verified,
        note_revision: 1,
        provenance: draft.provenance,
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_verified_knowledge_note_v1(&note).map_err(invalid_draft)?;
    Ok(note)
}

fn invalid_draft(_: KnowledgeValidationErrorV1) -> KnowledgeNoteCreationErrorV1 {
    KnowledgeNoteCreationErrorV1::InvalidDraft
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KnowledgeNoteProvenanceV1, KnowledgeNoteSourceBasisV1, KnowledgeNoteTimestampV1,
        KnowledgeNoteTopicHintV1, knowledge_note_creation_fingerprint_v1,
    };

    fn draft() -> ReviewedCandidateKnowledgeNoteDraftV1 {
        ReviewedCandidateKnowledgeNoteDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            provenance: KnowledgeNoteProvenanceV1 {
                approved_candidate_id: [1; 16],
                candidate_digest: [2; 32],
                source_evidence_id: [3; 16],
                source_evidence_revision: 4,
                review_id: [5; 16],
                decision_revision: 6,
                decided_by_owner_device_id: [7; 16],
            },
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount\nPayment by Friday".to_owned(),
            topic_hints: vec![
                KnowledgeNoteTopicHintV1::Financial,
                KnowledgeNoteTopicHintV1::Legal,
            ],
            source_basis: KnowledgeNoteSourceBasisV1::Combined,
            confidence_basis_points: 8_300,
            created_at: KnowledgeNoteTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 3,
            },
        }
    }

    #[test]
    fn reviewed_candidate_creates_exactly_one_deterministic_verified_note() {
        let first =
            create_verified_knowledge_note_from_reviewed_candidate_v1(draft()).expect("note");
        let second =
            create_verified_knowledge_note_from_reviewed_candidate_v1(draft()).expect("note");
        assert_eq!(first, second);
        assert_eq!(first.status, VerifiedKnowledgeNoteStatusV1::Verified);
        assert_eq!(first.note_revision, 1);
    }

    #[test]
    fn fingerprint_detects_conflicting_candidate_content() {
        let first = knowledge_note_creation_fingerprint_v1(&draft()).expect("fingerprint");
        let mut changed = draft();
        changed.title = "Другой заголовок".to_owned();
        let second = knowledge_note_creation_fingerprint_v1(&changed).expect("fingerprint");
        assert_ne!(first, second);
    }

    #[test]
    fn topic_hints_remain_classification_without_foreign_domain_identity() {
        let note =
            create_verified_knowledge_note_from_reviewed_candidate_v1(draft()).expect("note");
        assert_eq!(
            note.topic_hints,
            vec![
                KnowledgeNoteTopicHintV1::Financial,
                KnowledgeNoteTopicHintV1::Legal,
            ]
        );
    }

    #[test]
    fn missing_human_decision_evidence_is_rejected() {
        let mut invalid = draft();
        invalid.provenance.decided_by_owner_device_id = [0; 16];
        assert_eq!(
            create_verified_knowledge_note_from_reviewed_candidate_v1(invalid),
            Err(KnowledgeNoteCreationErrorV1::InvalidDraft)
        );
    }

    #[test]
    fn unordered_hints_and_invalid_confidence_are_rejected() {
        let mut invalid = draft();
        invalid.topic_hints.reverse();
        assert_eq!(
            create_verified_knowledge_note_from_reviewed_candidate_v1(invalid),
            Err(KnowledgeNoteCreationErrorV1::InvalidDraft)
        );

        let mut invalid = draft();
        invalid.confidence_basis_points = 0;
        assert_eq!(
            create_verified_knowledge_note_from_reviewed_candidate_v1(invalid),
            Err(KnowledgeNoteCreationErrorV1::InvalidDraft)
        );
    }
}
