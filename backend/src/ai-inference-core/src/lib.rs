#![forbid(unsafe_code)]

use hermes_ai_contracts::{
    AiContractValidationErrorV1, validate_provider_reply_generation_result_v1,
    validate_reply_inference_request_v1, validate_reply_inference_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceReceiptV1, AiInferenceTerminalStatusV1,
        AiPrivateSourceReceiptV1, AiProviderReplyGenerationResultV1, AiReplyLanguageV1,
        AiReplyToneV1, CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1,
    },
};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "hermes-ai-inference-core";
pub const AI_INFERENCE_PROVIDER_POLICY_REVISION_V1: u32 = 1;
const AI_REPLY_PROMPT_POLICY_V1: &[u8] = b"hermes-ai-reply-prompt-policy-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiInferenceRunStateV1 {
    Accepted,
    Executing,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiInferenceRunV1 {
    pub request: CommunicationReplySuggestionInferenceRequestV1,
    pub revision: u64,
    pub state: AiInferenceRunStateV1,
    pub terminal_result: Option<CommunicationReplySuggestionInferenceResultV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiInferenceExecutionPlanV1 {
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub request_digest: [u8; 32],
    pub source: AiPrivateSourceReceiptV1,
    pub tone: i32,
    pub language: i32,
    pub subject_policy: i32,
    pub maximum_output_bytes: u32,
    pub maximum_output_tokens: u32,
    pub egress_policy: i32,
    pub egress_policy_revision: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiInferenceCoreErrorV1 {
    InvalidRequest,
    RevisionConflict,
    InvalidTransition,
    InvalidProviderResult,
    InvalidResult,
}

pub fn accept_reply_inference_v1(
    request: CommunicationReplySuggestionInferenceRequestV1,
) -> Result<AiInferenceRunV1, AiInferenceCoreErrorV1> {
    validate_reply_inference_request_v1(&request).map_err(contract_request_error)?;
    Ok(AiInferenceRunV1 {
        request,
        revision: 1,
        state: AiInferenceRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_reply_inference_v1(
    run: &AiInferenceRunV1,
    expected_revision: u64,
) -> Result<(AiInferenceRunV1, AiInferenceExecutionPlanV1), AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if run.state != AiInferenceRunStateV1::Accepted || run.terminal_result.is_some() {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let source = run
        .request
        .source
        .clone()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let next = AiInferenceRunV1 {
        request: run.request.clone(),
        revision: run.revision + 1,
        state: AiInferenceRunStateV1::Executing,
        terminal_result: None,
    };
    Ok((
        next,
        AiInferenceExecutionPlanV1 {
            run_id: id16(&run.request.run_id)?,
            logical_owner_id: run.request.logical_owner_id.clone(),
            request_digest: id32(&context.request_digest)?,
            source,
            tone: run.request.tone,
            language: run.request.language,
            subject_policy: run.request.subject_policy,
            maximum_output_bytes: run.request.maximum_output_bytes,
            maximum_output_tokens: run.request.maximum_output_tokens,
            egress_policy: run.request.egress_policy,
            egress_policy_revision: run.request.egress_policy_revision,
        },
    ))
}

pub fn complete_reply_inference_v1(
    run: &AiInferenceRunV1,
    expected_revision: u64,
    provider_result: AiProviderReplyGenerationResultV1,
    provider_settings_revision: u64,
) -> Result<AiInferenceRunV1, AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if run.state != AiInferenceRunStateV1::Executing
        || run.terminal_result.is_some()
        || provider_settings_revision == 0
        || provider_result.request_id != run.request.run_id
    {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    validate_provider_reply_generation_result_v1(&provider_result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidProviderResult)?;
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let source = run
        .request
        .source
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let status = AiInferenceTerminalStatusV1::try_from(provider_result.terminal_status)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidProviderResult)?;
    let ready = status == AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady;
    let result = CommunicationReplySuggestionInferenceResultV1 {
        run_id: run.request.run_id.clone(),
        request_digest: context.request_digest.clone(),
        source_sha256: source.sha256.clone(),
        subject_utf8: provider_result.subject_utf8,
        body_utf8: provider_result.body_utf8,
        resolved_tone: provider_result.resolved_tone,
        resolved_language: provider_result.resolved_language,
        inference_receipt: ready.then(|| AiInferenceReceiptV1 {
            model_revision_sha256: provider_result.model_revision_sha256,
            prompt_policy_sha256: prompt_policy_sha256_v1().to_vec(),
            provider_settings_revision,
            provider_policy_revision: AI_INFERENCE_PROVIDER_POLICY_REVISION_V1,
        }),
        completeness: provider_result.completeness,
        confidence_basis_points: provider_result.confidence_basis_points,
        terminal_status: status as i32,
    };
    validate_reply_inference_result_v1(&result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidResult)?;
    Ok(AiInferenceRunV1 {
        request: run.request.clone(),
        revision: run.revision + 1,
        state: if ready {
            AiInferenceRunStateV1::Ready
        } else {
            AiInferenceRunStateV1::Rejected
        },
        terminal_result: Some(result),
    })
}

pub fn reject_reply_inference_v1(
    run: &AiInferenceRunV1,
    expected_revision: u64,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Result<AiInferenceRunV1, AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if !matches!(
        run.state,
        AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing
    ) || run.terminal_result.is_some()
        || !matches!(
            terminal_status,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected
        )
    {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let source = run
        .request
        .source
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let result = CommunicationReplySuggestionInferenceResultV1 {
        run_id: run.request.run_id.clone(),
        request_digest: context.request_digest.clone(),
        source_sha256: source.sha256.clone(),
        subject_utf8: Vec::new(),
        body_utf8: Vec::new(),
        resolved_tone: AiReplyToneV1::AiReplyToneUnspecified as i32,
        resolved_language: AiReplyLanguageV1::AiReplyLanguageUnspecified as i32,
        inference_receipt: None,
        completeness: AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32,
        confidence_basis_points: 0,
        terminal_status: terminal_status as i32,
    };
    validate_reply_inference_result_v1(&result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidResult)?;
    Ok(AiInferenceRunV1 {
        request: run.request.clone(),
        revision: run.revision + 1,
        state: AiInferenceRunStateV1::Rejected,
        terminal_result: Some(result),
    })
}

#[must_use]
pub fn prompt_policy_sha256_v1() -> [u8; 32] {
    Sha256::digest(AI_REPLY_PROMPT_POLICY_V1).into()
}

fn require_revision(
    run: &AiInferenceRunV1,
    expected_revision: u64,
) -> Result<(), AiInferenceCoreErrorV1> {
    if expected_revision == 0 || run.revision != expected_revision {
        return Err(AiInferenceCoreErrorV1::RevisionConflict);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], AiInferenceCoreErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)
}

fn id32(value: &[u8]) -> Result<[u8; 32], AiInferenceCoreErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)
}

fn contract_request_error(_: AiContractValidationErrorV1) -> AiInferenceCoreErrorV1 {
    AiInferenceCoreErrorV1::InvalidRequest
}

#[cfg(test)]
mod tests {
    use hermes_ai_contracts::{
        AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
        AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_OUTPUT_TOKENS_V1,
        seal_reply_inference_request_v1,
        wire::{
            AiContextReceiptV1, AiEgressPolicyV1, AiPrivateSourceReceiptV1,
            AiProviderReplyGenerationResultV1, AiReplyLanguageV1, AiReplySubjectPolicyV1,
            AiReplyToneV1, AiUseCaseV1,
        },
    };

    use super::*;

    fn accepted() -> AiInferenceRunV1 {
        let request =
            seal_reply_inference_request_v1(CommunicationReplySuggestionInferenceRequestV1 {
                run_id: vec![1; 16],
                context: Some(AiContextReceiptV1 {
                    context_id: vec![2; 16],
                    use_case: AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32,
                    source_evidence_id: vec![3; 16],
                    source_evidence_revision: 4,
                    contract_major: AI_CONTRACT_MAJOR_V1,
                    contract_revision: AI_CONTRACT_REVISION_V1,
                    contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                    request_digest: Vec::new(),
                }),
                source: Some(AiPrivateSourceReceiptV1 {
                    reference_id: vec![5; 16],
                    declared_bytes: 6,
                    sha256: vec![7; 32],
                    custody_transfer_source_proof: vec![8; 64],
                }),
                tone: AiReplyToneV1::AiReplyToneWarm as i32,
                language: AiReplyLanguageV1::AiReplyLanguageAuto as i32,
                subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
                maximum_output_bytes: 4_096,
                maximum_output_tokens: 512,
                egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
                egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
                logical_owner_id: "owner-1".to_owned(),
            })
            .expect("request");
        accept_reply_inference_v1(request).expect("accepted")
    }

    #[test]
    fn lifecycle_is_revision_fenced_and_terminal() {
        let accepted = accepted();
        let (executing, plan) = begin_reply_inference_v1(&accepted, 1).expect("executing");
        assert_eq!(plan.maximum_output_tokens, 512);
        let ready = complete_reply_inference_v1(
            &executing,
            2,
            AiProviderReplyGenerationResultV1 {
                request_id: vec![1; 16],
                subject_utf8: b"Re: subject".to_vec(),
                body_utf8: b"Suggested reply".to_vec(),
                resolved_tone: AiReplyToneV1::AiReplyToneWarm as i32,
                resolved_language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
                model_revision_sha256: vec![9; 32],
                input_tokens: 12,
                output_tokens: 8,
                terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
                completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
                confidence_basis_points: 8_000,
            },
            11,
        )
        .expect("ready");
        assert_eq!(ready.state, AiInferenceRunStateV1::Ready);
        assert_eq!(
            ready
                .terminal_result
                .as_ref()
                .expect("result")
                .confidence_basis_points,
            8_000
        );
        assert_eq!(
            begin_reply_inference_v1(&ready, 3),
            Err(AiInferenceCoreErrorV1::InvalidTransition)
        );
    }

    #[test]
    fn policy_rejection_has_no_candidate_or_provider_receipt() {
        let rejected = reject_reply_inference_v1(
            &accepted(),
            1,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy,
        )
        .expect("rejected");
        let result = rejected.terminal_result.expect("result");
        assert!(result.body_utf8.is_empty());
        assert!(result.inference_receipt.is_none());
    }

    #[test]
    fn core_has_one_fixed_prompt_policy_and_no_provider_selector() {
        assert_ne!(prompt_policy_sha256_v1(), [0; 32]);
        assert!(AI_MAX_OUTPUT_TOKENS_V1 >= 512);
    }
}
