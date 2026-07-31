use crate::model::validate_draft;
use crate::{
    ReviewedCandidateTaskDraftV1, TaskStatusV1, TaskV1, TasksValidationErrorV1, derive_task_id_v1,
    validate_task_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCreationErrorV1 {
    InvalidDraft,
}

pub fn create_task_from_reviewed_candidate_v1(
    draft: ReviewedCandidateTaskDraftV1,
) -> Result<TaskV1, TaskCreationErrorV1> {
    validate_draft(&draft).map_err(invalid_draft)?;
    let task_id = derive_task_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )
    .map_err(invalid_draft)?;
    let task = TaskV1 {
        task_id,
        logical_owner_id: draft.logical_owner_id,
        title: draft.title,
        due_text_hint: draft.due_text_hint,
        assignee_label_hint: draft.assignee_label_hint,
        status: TaskStatusV1::Open,
        task_revision: 1,
        provenance: draft.provenance,
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_task_v1(&task).map_err(invalid_draft)?;
    Ok(task)
}

fn invalid_draft(_: TasksValidationErrorV1) -> TaskCreationErrorV1 {
    TaskCreationErrorV1::InvalidDraft
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskProvenanceV1, TaskTimestampV1, task_creation_fingerprint_v1};

    fn draft() -> ReviewedCandidateTaskDraftV1 {
        ReviewedCandidateTaskDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            provenance: TaskProvenanceV1 {
                approved_candidate_id: [1; 16],
                candidate_digest: [2; 32],
                source_evidence_id: [3; 16],
                source_evidence_revision: 4,
                review_id: [5; 16],
                decision_revision: 6,
                decided_by_owner_device_id: [7; 16],
            },
            title: "Подготовить отчёт".to_owned(),
            due_text_hint: Some("до пятницы".to_owned()),
            assignee_label_hint: Some("я".to_owned()),
            created_at: TaskTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 3,
            },
        }
    }

    #[test]
    fn reviewed_candidate_creates_exactly_one_deterministic_open_task() {
        let first = create_task_from_reviewed_candidate_v1(draft()).expect("task");
        let second = create_task_from_reviewed_candidate_v1(draft()).expect("task");
        assert_eq!(first, second);
        assert_eq!(first.status, TaskStatusV1::Open);
        assert_eq!(first.task_revision, 1);
    }

    #[test]
    fn fingerprint_detects_conflicting_candidate_content() {
        let first = task_creation_fingerprint_v1(&draft()).expect("fingerprint");
        let mut changed = draft();
        changed.title = "Другой заголовок".to_owned();
        let second = task_creation_fingerprint_v1(&changed).expect("fingerprint");
        assert_ne!(first, second);
    }

    #[test]
    fn hints_do_not_materialize_foreign_domain_identity() {
        let task = create_task_from_reviewed_candidate_v1(draft()).expect("task");
        assert_eq!(task.due_text_hint.as_deref(), Some("до пятницы"));
        assert_eq!(task.assignee_label_hint.as_deref(), Some("я"));
    }

    #[test]
    fn missing_human_decision_evidence_is_rejected() {
        let mut invalid = draft();
        invalid.provenance.decided_by_owner_device_id = [0; 16];
        assert_eq!(
            create_task_from_reviewed_candidate_v1(invalid),
            Err(TaskCreationErrorV1::InvalidDraft)
        );
    }
}
