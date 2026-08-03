use crate::{
    SpeechToTextRejectionV1, SpeechToTextRequestV1, SpeechToTextResultV1, SpeechToTextTerminalV1,
    validate_speech_to_text_request_v1, validate_speech_to_text_result_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextRunStateV1 {
    Accepted,
    Executing,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextRunV1 {
    pub request: SpeechToTextRequestV1,
    pub state: SpeechToTextRunStateV1,
    pub revision: u64,
    pub terminal_result: Option<SpeechToTextResultV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextCoreErrorV1 {
    InvalidRequest,
    InvalidResult,
    RevisionConflict,
    InvalidTransition,
}

pub fn accept_speech_to_text_v1(
    request: SpeechToTextRequestV1,
) -> Result<SpeechToTextRunV1, SpeechToTextCoreErrorV1> {
    validate_speech_to_text_request_v1(&request)
        .map_err(|_| SpeechToTextCoreErrorV1::InvalidRequest)?;
    Ok(SpeechToTextRunV1 {
        request,
        state: SpeechToTextRunStateV1::Accepted,
        revision: 1,
        terminal_result: None,
    })
}

pub fn begin_speech_to_text_v1(
    current: &SpeechToTextRunV1,
    expected_revision: u64,
) -> Result<SpeechToTextRunV1, SpeechToTextCoreErrorV1> {
    validate_speech_to_text_run_v1(current)?;
    require_revision(current, expected_revision)?;
    if current.state != SpeechToTextRunStateV1::Accepted {
        return Err(SpeechToTextCoreErrorV1::InvalidTransition);
    }
    Ok(SpeechToTextRunV1 {
        request: current.request.clone(),
        state: SpeechToTextRunStateV1::Executing,
        revision: next_revision(current.revision)?,
        terminal_result: None,
    })
}

pub fn complete_speech_to_text_v1(
    current: &SpeechToTextRunV1,
    expected_revision: u64,
    result: SpeechToTextResultV1,
) -> Result<SpeechToTextRunV1, SpeechToTextCoreErrorV1> {
    validate_speech_to_text_run_v1(current)?;
    require_revision(current, expected_revision)?;
    if current.state != SpeechToTextRunStateV1::Executing
        || !matches!(result.terminal, SpeechToTextTerminalV1::Ready(_))
    {
        return Err(SpeechToTextCoreErrorV1::InvalidTransition);
    }
    validate_speech_to_text_result_v1(&current.request, &result)
        .map_err(|_| SpeechToTextCoreErrorV1::InvalidResult)?;
    Ok(SpeechToTextRunV1 {
        request: current.request.clone(),
        state: SpeechToTextRunStateV1::Ready,
        revision: next_revision(current.revision)?,
        terminal_result: Some(result),
    })
}

pub fn reject_speech_to_text_v1(
    current: &SpeechToTextRunV1,
    expected_revision: u64,
    rejection: SpeechToTextRejectionV1,
) -> Result<SpeechToTextRunV1, SpeechToTextCoreErrorV1> {
    validate_speech_to_text_run_v1(current)?;
    require_revision(current, expected_revision)?;
    if !matches!(
        current.state,
        SpeechToTextRunStateV1::Accepted | SpeechToTextRunStateV1::Executing
    ) {
        return Err(SpeechToTextCoreErrorV1::InvalidTransition);
    }
    let result = SpeechToTextResultV1 {
        request_id: current.request.request_id,
        request_digest: current.request.request_digest,
        source_sha256: current.request.source.sha256,
        terminal: SpeechToTextTerminalV1::Rejected(rejection),
    };
    validate_speech_to_text_result_v1(&current.request, &result)
        .map_err(|_| SpeechToTextCoreErrorV1::InvalidResult)?;
    Ok(SpeechToTextRunV1 {
        request: current.request.clone(),
        state: SpeechToTextRunStateV1::Rejected,
        revision: next_revision(current.revision)?,
        terminal_result: Some(result),
    })
}

pub fn validate_speech_to_text_run_v1(
    run: &SpeechToTextRunV1,
) -> Result<(), SpeechToTextCoreErrorV1> {
    validate_speech_to_text_request_v1(&run.request)
        .map_err(|_| SpeechToTextCoreErrorV1::InvalidRequest)?;
    if run.revision == 0 {
        return Err(SpeechToTextCoreErrorV1::InvalidRequest);
    }
    match (run.state, run.terminal_result.as_ref()) {
        (SpeechToTextRunStateV1::Accepted | SpeechToTextRunStateV1::Executing, None) => Ok(()),
        (SpeechToTextRunStateV1::Ready, Some(result))
            if matches!(result.terminal, SpeechToTextTerminalV1::Ready(_)) =>
        {
            validate_speech_to_text_result_v1(&run.request, result)
                .map_err(|_| SpeechToTextCoreErrorV1::InvalidResult)
        }
        (SpeechToTextRunStateV1::Rejected, Some(result))
            if matches!(result.terminal, SpeechToTextTerminalV1::Rejected(_)) =>
        {
            validate_speech_to_text_result_v1(&run.request, result)
                .map_err(|_| SpeechToTextCoreErrorV1::InvalidResult)
        }
        _ => Err(SpeechToTextCoreErrorV1::InvalidTransition),
    }
}

fn require_revision(
    current: &SpeechToTextRunV1,
    expected_revision: u64,
) -> Result<(), SpeechToTextCoreErrorV1> {
    if expected_revision == 0 || current.revision != expected_revision {
        return Err(SpeechToTextCoreErrorV1::RevisionConflict);
    }
    Ok(())
}

fn next_revision(current: u64) -> Result<u64, SpeechToTextCoreErrorV1> {
    current
        .checked_add(1)
        .ok_or(SpeechToTextCoreErrorV1::RevisionConflict)
}

#[cfg(test)]
mod tests {
    use crate::{
        SpeechAudioFormatV1, SpeechBlobReceiptV1, SpeechLanguageV1, SpeechToTextExecutionReceiptV1,
        SpeechTranscriptArtifactV1, SpeechTranscriptCompletenessV1,
    };

    use super::*;

    fn request() -> SpeechToTextRequestV1 {
        SpeechToTextRequestV1 {
            request_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: SpeechBlobReceiptV1 {
                reference_id: [2; 16],
                declared_bytes: 32_044,
                sha256: [3; 32],
                custody_proof: vec![4; 32],
            },
            audio_format: SpeechAudioFormatV1::WavPcmS16LeMono16Khz,
            duration_millis: 1_000,
            requested_language: SpeechLanguageV1::Auto,
            consent_receipt_id: [5; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 128,
            request_digest: [6; 32],
        }
    }

    #[test]
    fn lifecycle_requires_accept_execute_ready_order() {
        let accepted = accept_speech_to_text_v1(request()).expect("accepted");
        let executing = begin_speech_to_text_v1(&accepted, 1).expect("executing");
        let result = SpeechToTextResultV1 {
            request_id: accepted.request.request_id,
            request_digest: accepted.request.request_digest,
            source_sha256: accepted.request.source.sha256,
            terminal: SpeechToTextTerminalV1::Ready(SpeechTranscriptArtifactV1 {
                receipt: SpeechBlobReceiptV1 {
                    reference_id: [7; 16],
                    declared_bytes: 4_096,
                    sha256: [8; 32],
                    custody_proof: vec![9; 32],
                },
                detected_language: SpeechLanguageV1::Russian,
                segment_count: 12,
                completeness: SpeechTranscriptCompletenessV1::Complete,
                confidence_basis_points: 9_100,
                execution_receipt: SpeechToTextExecutionReceiptV1 {
                    provider_contract_schema_sha256: [10; 32],
                    model_revision_sha256: [11; 32],
                    provider_settings_revision: 2,
                    provider_policy_revision: 1,
                },
            }),
        };
        let ready = complete_speech_to_text_v1(&executing, 2, result).expect("ready");
        assert_eq!(ready.state, SpeechToTextRunStateV1::Ready);
        assert_eq!(ready.revision, 3);
    }

    #[test]
    fn accepted_run_can_fail_closed_before_provider_execution() {
        let accepted = accept_speech_to_text_v1(request()).expect("accepted");
        let rejected = reject_speech_to_text_v1(
            &accepted,
            accepted.revision,
            SpeechToTextRejectionV1::ConsentRejected,
        )
        .expect("rejected");
        assert_eq!(rejected.state, SpeechToTextRunStateV1::Rejected);
    }
}
