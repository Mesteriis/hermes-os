use super::constants::{ARTIFACT_ID_PREFIX, JOB_ID_PREFIX};
use super::models::{DocumentArtifactKind, DocumentProcessingStep};

pub(super) fn job_id(document_id: &str, step: DocumentProcessingStep) -> String {
    format!("{JOB_ID_PREFIX}{document_id}:{:0}", step.as_str())
}

pub(super) fn artifact_id(document_id: &str, artifact_kind: DocumentArtifactKind) -> String {
    format!(
        "{ARTIFACT_ID_PREFIX}{document_id}:{:0}",
        artifact_kind.as_str()
    )
}
