#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttRunStateV1 {
    Accepted,
    Executing,
    Ready,
    Rejected,
    Uncertain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttRunIdentityV1 {
    pub logical_owner_id: String,
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub source_reference_id: [u8; 16],
    pub source_declared_bytes: u64,
    pub source_sha256: [u8; 32],
    pub model_revision_sha256: [u8; 32],
    pub provider_settings_revision: u64,
    pub provider_policy_revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttReadyMetadataV1 {
    pub transcript_reference_id: [u8; 16],
    pub transcript_declared_bytes: u64,
    pub transcript_sha256: [u8; 32],
    pub detected_language: u32,
    pub segment_count: u32,
    pub completeness: u32,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedWhisperSttRunV1 {
    pub identity: WhisperSttRunIdentityV1,
    pub revision: u64,
    pub state: WhisperSttRunStateV1,
    pub ready: Option<WhisperSttReadyMetadataV1>,
    pub reject_code: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttTransitionV1 {
    pub current_revision: u64,
    pub next: PersistedWhisperSttRunV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttPersistenceOutcomeV1 {
    pub persisted: PersistedWhisperSttRunV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    RevisionConflict,
    InvalidTransition,
}

pub(crate) fn validate_identity(value: &WhisperSttRunIdentityV1) -> bool {
    valid_owner(&value.logical_owner_id)
        && value.request_id != [0; 16]
        && value.request_digest != [0; 32]
        && value.source_reference_id != [0; 16]
        && (1..=512 * 1024 * 1024).contains(&value.source_declared_bytes)
        && value.source_sha256 != [0; 32]
        && value.model_revision_sha256 != [0; 32]
        && value.provider_settings_revision > 0
        && value.provider_policy_revision > 0
}

pub(crate) fn validate_run(
    value: &PersistedWhisperSttRunV1,
) -> Result<(), WhisperSttPersistenceErrorV1> {
    if !validate_identity(&value.identity) || value.revision == 0 {
        return Err(WhisperSttPersistenceErrorV1::InvalidInput);
    }
    match (value.state, value.ready.as_ref(), value.reject_code) {
        (
            WhisperSttRunStateV1::Accepted
            | WhisperSttRunStateV1::Executing
            | WhisperSttRunStateV1::Uncertain,
            None,
            None,
        ) => Ok(()),
        (WhisperSttRunStateV1::Ready, Some(ready), None) if valid_ready(ready) => Ok(()),
        (WhisperSttRunStateV1::Rejected, None, Some(code)) if (1..=6).contains(&code) => Ok(()),
        _ => Err(WhisperSttPersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_accepted(
    value: &PersistedWhisperSttRunV1,
) -> Result<(), WhisperSttPersistenceErrorV1> {
    validate_run(value)?;
    if value.revision != 1 || value.state != WhisperSttRunStateV1::Accepted {
        return Err(WhisperSttPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_transition(
    current: &PersistedWhisperSttRunV1,
    transition: &WhisperSttTransitionV1,
) -> Result<(), WhisperSttPersistenceErrorV1> {
    validate_run(&transition.next)?;
    if current.revision != transition.current_revision
        || transition.next.revision != transition.current_revision + 1
        || current.identity != transition.next.identity
    {
        return Err(WhisperSttPersistenceErrorV1::RevisionConflict);
    }
    if !matches!(
        (current.state, transition.next.state),
        (
            WhisperSttRunStateV1::Accepted,
            WhisperSttRunStateV1::Executing
        ) | (
            WhisperSttRunStateV1::Accepted,
            WhisperSttRunStateV1::Rejected
        ) | (WhisperSttRunStateV1::Executing, WhisperSttRunStateV1::Ready)
            | (
                WhisperSttRunStateV1::Executing,
                WhisperSttRunStateV1::Rejected
            )
            | (
                WhisperSttRunStateV1::Executing,
                WhisperSttRunStateV1::Uncertain
            )
    ) {
        return Err(WhisperSttPersistenceErrorV1::InvalidTransition);
    }
    Ok(())
}

fn valid_ready(value: &WhisperSttReadyMetadataV1) -> bool {
    value.transcript_reference_id != [0; 16]
        && (1..=4 * 1024 * 1024).contains(&value.transcript_declared_bytes)
        && value.transcript_sha256 != [0; 32]
        && (1..=4).contains(&value.detected_language)
        && value.segment_count <= 100_000
        && (1..=2).contains(&value.completeness)
        && value.confidence_basis_points <= 10_000
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted() -> PersistedWhisperSttRunV1 {
        PersistedWhisperSttRunV1 {
            identity: WhisperSttRunIdentityV1 {
                logical_owner_id: "owner-1".to_owned(),
                request_id: [1; 16],
                request_digest: [2; 32],
                source_reference_id: [3; 16],
                source_declared_bytes: 44,
                source_sha256: [4; 32],
                model_revision_sha256: [5; 32],
                provider_settings_revision: 6,
                provider_policy_revision: 1,
            },
            revision: 1,
            state: WhisperSttRunStateV1::Accepted,
            ready: None,
            reject_code: None,
        }
    }

    #[test]
    fn exact_identity_and_revision_fence_every_transition() {
        let current = accepted();
        let mut executing = current.clone();
        executing.revision = 2;
        executing.state = WhisperSttRunStateV1::Executing;
        assert_eq!(
            validate_transition(
                &current,
                &WhisperSttTransitionV1 {
                    current_revision: 1,
                    next: executing.clone(),
                }
            ),
            Ok(())
        );
        executing.identity.request_digest = [9; 32];
        assert_eq!(
            validate_transition(
                &current,
                &WhisperSttTransitionV1 {
                    current_revision: 1,
                    next: executing,
                }
            ),
            Err(WhisperSttPersistenceErrorV1::RevisionConflict)
        );
    }

    #[test]
    fn ambiguous_execution_can_only_become_uncertain() {
        let mut executing = accepted();
        executing.revision = 2;
        executing.state = WhisperSttRunStateV1::Executing;
        let mut uncertain = executing.clone();
        uncertain.revision = 3;
        uncertain.state = WhisperSttRunStateV1::Uncertain;
        assert_eq!(
            validate_transition(
                &executing,
                &WhisperSttTransitionV1 {
                    current_revision: 2,
                    next: uncertain,
                }
            ),
            Ok(())
        );
    }
}
