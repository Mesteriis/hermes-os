use std::collections::BTreeSet;

use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_CUSTODY_PROOF_BYTES_V1,
    AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1, AI_MAX_EXPLANATION_REASONS_V1,
    AI_MAX_OUTPUT_TOKENS_V1, AI_MAX_PRIVATE_SOURCE_BYTES_V1, AI_MAX_SENDER_BYTES_V1,
    AI_MAX_SUBJECT_BYTES_V1,
    validation::AiContractValidationErrorV1,
    wire::{
        AiEgressPolicyV1, AiExplanationReasonKindV1, AiExplanationReasonV1,
        AiExplanationSourceBasisV1, AiExplanationSourceContentV1, AiInferenceCompletenessV1,
        AiInferenceReceiptV1, AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1,
        AiProviderExplanationRequestV1, AiProviderExplanationResultV1, AiUseCaseV1,
        CommunicationExplanationInferenceRequestV1, CommunicationExplanationInferenceResultV1,
    },
};

pub fn seal_explanation_inference_request_v1(
    mut request: CommunicationExplanationInferenceRequestV1,
) -> Result<CommunicationExplanationInferenceRequestV1, AiContractValidationErrorV1> {
    let digest = compute_explanation_inference_request_digest_v1(&request)?;
    request
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?
        .request_digest = digest.to_vec();
    validate_explanation_inference_request_v1(&request)?;
    Ok(request)
}

pub fn compute_explanation_inference_request_digest_v1(
    request: &CommunicationExplanationInferenceRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    let mut canonical = request.clone();
    canonical
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?
        .request_digest
        .clear();
    canonical
        .source
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidSource)?
        .custody_transfer_source_proof
        .clear();
    Ok(Sha256::digest(canonical.encode_to_vec()).into())
}

pub fn compute_provider_explanation_request_digest_v1(
    request: &AiProviderExplanationRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    validate_provider_explanation_request_v1(request)?;
    Ok(Sha256::digest(request.encode_to_vec()).into())
}

pub fn validate_explanation_inference_request_v1(
    request: &CommunicationExplanationInferenceRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.run_id)
        || !valid_owner(&request.logical_owner_id)
        || request.maximum_reasons != AI_MAX_EXPLANATION_REASONS_V1
        || request.maximum_reason_text_bytes != AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1
        || request.maximum_output_tokens == 0
        || request.maximum_output_tokens > AI_MAX_OUTPUT_TOKENS_V1
        || request.egress_policy != AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32
        || request.egress_policy_revision != AI_LOCAL_EGRESS_POLICY_REVISION_V1
    {
        return Err(AiContractValidationErrorV1::InvalidRequest);
    }
    let context = request
        .context
        .as_ref()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?;
    if !id16(&context.context_id)
        || context.use_case != AiUseCaseV1::AiUseCaseCommunicationExplanation as i32
        || !id16(&context.source_evidence_id)
        || context.source_evidence_revision == 0
        || context.contract_major != AI_CONTRACT_MAJOR_V1
        || context.contract_revision != AI_CONTRACT_REVISION_V1
        || context.contract_schema_sha256 != AI_CONTRACTS_SCHEMA_SHA256
        || !sha256(&context.request_digest)
        || context.request_digest != compute_explanation_inference_request_digest_v1(request)?
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    validate_private_source(request.source.as_ref())
}

pub fn validate_explanation_source_content_v1(
    content: &AiExplanationSourceContentV1,
) -> Result<(), AiContractValidationErrorV1> {
    if content.sender_utf8.len() > AI_MAX_SENDER_BYTES_V1
        || content.subject_utf8.len() > AI_MAX_SUBJECT_BYTES_V1
        || content.body_utf8.is_empty()
        || content.body_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || content.encoded_len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(&content.sender_utf8).is_err()
        || std::str::from_utf8(&content.subject_utf8).is_err()
        || std::str::from_utf8(&content.body_utf8).is_err()
    {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(())
}

pub fn encode_explanation_source_content_v1(
    content: &AiExplanationSourceContentV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_explanation_source_content_v1(content)?;
    Ok(content.encode_to_vec())
}

pub fn decode_explanation_source_content_v1(
    bytes: &[u8],
) -> Result<AiExplanationSourceContentV1, AiContractValidationErrorV1> {
    let content = AiExplanationSourceContentV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidSource)?;
    validate_explanation_source_content_v1(&content)?;
    if content.encode_to_vec() != bytes {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(content)
}

pub fn validate_explanation_inference_result_v1(
    result: &CommunicationExplanationInferenceResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.run_id) || !sha256(&result.request_digest) || !sha256(&result.source_sha256) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    validate_reasons(&result.reasons)?;
    if !valid_inference_receipt(result.inference_receipt.as_ref())
        || !valid_completeness(result.completeness)
        || result.confidence_basis_points > 10_000
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(())
}

pub fn encode_explanation_inference_result_v1(
    result: &CommunicationExplanationInferenceResultV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_explanation_inference_result_v1(result)?;
    Ok(result.encode_to_vec())
}

pub fn decode_explanation_inference_result_v1(
    bytes: &[u8],
) -> Result<CommunicationExplanationInferenceResultV1, AiContractValidationErrorV1> {
    let result = CommunicationExplanationInferenceResultV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    validate_explanation_inference_result_v1(&result)?;
    if result.encode_to_vec() != bytes {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(result)
}

pub fn validate_provider_explanation_request_v1(
    request: &AiProviderExplanationRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.request_id)
        || request.input_utf8.is_empty()
        || request.input_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(&request.input_utf8).is_err()
        || request.maximum_reasons != AI_MAX_EXPLANATION_REASONS_V1
        || request.maximum_reason_text_bytes != AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1
        || request.maximum_output_tokens == 0
        || request.maximum_output_tokens > AI_MAX_OUTPUT_TOKENS_V1
        || request.egress_policy != AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32
        || request.egress_policy_revision != AI_LOCAL_EGRESS_POLICY_REVISION_V1
    {
        return Err(AiContractValidationErrorV1::InvalidPolicy);
    }
    Ok(())
}

pub fn validate_provider_explanation_result_v1(
    result: &AiProviderExplanationResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.request_id) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if provider_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    validate_reasons(&result.reasons)?;
    if !sha256(&result.model_revision_sha256)
        || result.output_tokens == 0
        || result.output_tokens > AI_MAX_OUTPUT_TOKENS_V1
        || !valid_completeness(result.completeness)
        || result.confidence_basis_points > 10_000
        || result.provider_settings_revision == 0
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(())
}

pub fn encode_provider_explanation_result_v1(
    result: &AiProviderExplanationResultV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_provider_explanation_result_v1(result)?;
    Ok(result.encode_to_vec())
}

pub fn decode_provider_explanation_result_v1(
    bytes: &[u8],
) -> Result<AiProviderExplanationResultV1, AiContractValidationErrorV1> {
    let result = AiProviderExplanationResultV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    validate_provider_explanation_result_v1(&result)?;
    if result.encode_to_vec() != bytes {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(result)
}

fn validate_reasons(reasons: &[AiExplanationReasonV1]) -> Result<(), AiContractValidationErrorV1> {
    if reasons.len() > AI_MAX_EXPLANATION_REASONS_V1 as usize {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let mut kinds = BTreeSet::new();
    for reason in reasons {
        if !valid_reason_kind(reason.kind)
            || !valid_source_basis(reason.source_basis)
            || reason.explanation_utf8.is_empty()
            || reason.explanation_utf8.len() > AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1 as usize
            || std::str::from_utf8(&reason.explanation_utf8).is_err()
            || reason.confidence_basis_points > 10_000
            || !kinds.insert(reason.kind)
        {
            return Err(AiContractValidationErrorV1::InvalidResult);
        }
    }
    Ok(())
}

fn validate_private_source(
    source: Option<&AiPrivateSourceReceiptV1>,
) -> Result<(), AiContractValidationErrorV1> {
    let source = source.ok_or(AiContractValidationErrorV1::InvalidSource)?;
    if !id16(&source.reference_id)
        || !(1..=AI_MAX_PRIVATE_SOURCE_BYTES_V1).contains(&source.declared_bytes)
        || !sha256(&source.sha256)
        || source.custody_transfer_source_proof.is_empty()
        || source.custody_transfer_source_proof.len() > AI_MAX_CUSTODY_PROOF_BYTES_V1
    {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(())
}

fn rejection_is_sanitized(
    result: &CommunicationExplanationInferenceResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.reasons.is_empty()
        && result.inference_receipt.is_none()
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
}

fn provider_rejection_is_sanitized(
    result: &AiProviderExplanationResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.reasons.is_empty()
        && result.model_revision_sha256.is_empty()
        && result.input_tokens == 0
        && result.output_tokens == 0
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
        && result.provider_settings_revision == 0
}

fn valid_inference_receipt(receipt: Option<&AiInferenceReceiptV1>) -> bool {
    receipt.is_some_and(|receipt| {
        sha256(&receipt.model_revision_sha256)
            && sha256(&receipt.prompt_policy_sha256)
            && receipt.provider_settings_revision > 0
            && receipt.provider_policy_revision > 0
    })
}

fn valid_reason_kind(value: i32) -> bool {
    matches!(
        AiExplanationReasonKindV1::try_from(value),
        Ok(AiExplanationReasonKindV1::AiExplanationReasonKindUrgency
            | AiExplanationReasonKindV1::AiExplanationReasonKindFinancialAttention
            | AiExplanationReasonKindV1::AiExplanationReasonKindLegalOrContractual
            | AiExplanationReasonKindV1::AiExplanationReasonKindReplyRequested
            | AiExplanationReasonKindV1::AiExplanationReasonKindDeadline
            | AiExplanationReasonKindV1::AiExplanationReasonKindAttachmentReference
            | AiExplanationReasonKindV1::AiExplanationReasonKindMarketingOrBulk
            | AiExplanationReasonKindV1::AiExplanationReasonKindOtherAttention)
    )
}

fn valid_source_basis(value: i32) -> bool {
    matches!(
        AiExplanationSourceBasisV1::try_from(value),
        Ok(AiExplanationSourceBasisV1::AiExplanationSourceBasisSubject
            | AiExplanationSourceBasisV1::AiExplanationSourceBasisBody
            | AiExplanationSourceBasisV1::AiExplanationSourceBasisCanonicalMetadata
            | AiExplanationSourceBasisV1::AiExplanationSourceBasisCombined)
    )
}

fn valid_completeness(value: i32) -> bool {
    matches!(
        AiInferenceCompletenessV1::try_from(value),
        Ok(AiInferenceCompletenessV1::AiInferenceCompletenessComplete
            | AiInferenceCompletenessV1::AiInferenceCompletenessPartial)
    )
}

fn rejection_status(status: AiInferenceTerminalStatusV1) -> bool {
    matches!(
        status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected
    )
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(value: &[u8]) -> bool {
    value.len() == 16 && value.iter().any(|byte| *byte != 0)
}

fn sha256(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{AiContextReceiptV1, AiPrivateSourceReceiptV1};

    fn reason(kind: AiExplanationReasonKindV1) -> AiExplanationReasonV1 {
        AiExplanationReasonV1 {
            kind: kind as i32,
            explanation_utf8: b"A bounded attention reason.".to_vec(),
            source_basis: AiExplanationSourceBasisV1::AiExplanationSourceBasisCombined as i32,
            confidence_basis_points: 8_000,
        }
    }

    #[test]
    fn explanation_is_a_distinct_sealed_use_case_with_fixed_taxonomy_budgets() {
        let request = CommunicationExplanationInferenceRequestV1 {
            run_id: vec![41; 16],
            context: Some(AiContextReceiptV1 {
                context_id: vec![42; 16],
                use_case: AiUseCaseV1::AiUseCaseCommunicationExplanation as i32,
                source_evidence_id: vec![43; 16],
                source_evidence_revision: 5,
                contract_major: AI_CONTRACT_MAJOR_V1,
                contract_revision: AI_CONTRACT_REVISION_V1,
                contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                request_digest: Vec::new(),
            }),
            source: Some(AiPrivateSourceReceiptV1 {
                reference_id: vec![44; 16],
                declared_bytes: 128,
                sha256: vec![45; 32],
                custody_transfer_source_proof: vec![46; 64],
            }),
            maximum_reasons: AI_MAX_EXPLANATION_REASONS_V1,
            maximum_reason_text_bytes: AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
            logical_owner_id: "owner-1".to_owned(),
        };
        let sealed = seal_explanation_inference_request_v1(request).expect("explanation");
        validate_explanation_inference_request_v1(&sealed).expect("valid");
        let mut changed_budget = sealed;
        changed_budget.maximum_reasons -= 1;
        assert_eq!(
            validate_explanation_inference_request_v1(&changed_budget),
            Err(AiContractValidationErrorV1::InvalidRequest)
        );
    }

    #[test]
    fn provider_result_rejects_duplicate_reason_kinds() {
        let duplicate = reason(AiExplanationReasonKindV1::AiExplanationReasonKindDeadline);
        let result = AiProviderExplanationResultV1 {
            request_id: vec![51; 16],
            reasons: vec![duplicate.clone(), duplicate],
            model_revision_sha256: vec![52; 32],
            input_tokens: 10,
            output_tokens: 20,
            terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
            confidence_basis_points: 8_000,
            provider_settings_revision: 3,
        };
        assert_eq!(
            validate_provider_explanation_result_v1(&result),
            Err(AiContractValidationErrorV1::InvalidResult)
        );
    }

    #[test]
    fn empty_reason_result_is_valid_and_does_not_fabricate_attention() {
        let result = AiProviderExplanationResultV1 {
            request_id: vec![61; 16],
            reasons: Vec::new(),
            model_revision_sha256: vec![62; 32],
            input_tokens: 10,
            output_tokens: 1,
            terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
            confidence_basis_points: 10_000,
            provider_settings_revision: 3,
        };
        validate_provider_explanation_result_v1(&result).expect("empty candidate is explicit");
    }
}
