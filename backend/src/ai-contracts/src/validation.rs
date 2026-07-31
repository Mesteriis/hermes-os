use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_CUSTODY_PROOF_BYTES_V1, AI_MAX_OUTPUT_BYTES_V1,
    AI_MAX_OUTPUT_TOKENS_V1, AI_MAX_PRIVATE_SOURCE_BYTES_V1, AI_MAX_SENDER_BYTES_V1,
    AI_MAX_SUBJECT_BYTES_V1,
    wire::{
        AiEgressPolicyV1, AiInferenceCompletenessV1, AiInferenceReceiptV1,
        AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1, AiProviderReplyGenerationRequestV1,
        AiProviderReplyGenerationResultV1, AiProviderSummaryGenerationRequestV1,
        AiProviderSummaryGenerationResultV1, AiReplyLanguageV1, AiReplySourceContentV1,
        AiReplySubjectPolicyV1, AiReplyToneV1, AiSummaryLanguageV1, AiSummaryLengthV1,
        AiSummarySourceContentV1, AiUseCaseV1, CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1, CommunicationSummaryInferenceRequestV1,
        CommunicationSummaryInferenceResultV1,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiContractValidationErrorV1 {
    InvalidRequest,
    InvalidReceipt,
    InvalidSource,
    InvalidPolicy,
    InvalidResult,
}

pub fn seal_reply_inference_request_v1(
    mut request: CommunicationReplySuggestionInferenceRequestV1,
) -> Result<CommunicationReplySuggestionInferenceRequestV1, AiContractValidationErrorV1> {
    let digest = compute_reply_inference_request_digest_v1(&request)?;
    let context = request
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?;
    context.request_digest = digest.to_vec();
    validate_reply_inference_request_v1(&request)?;
    Ok(request)
}

pub fn compute_reply_inference_request_digest_v1(
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    let mut canonical = request.clone();
    let context = canonical
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?;
    context.request_digest.clear();
    canonical
        .source
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidSource)?
        .custody_transfer_source_proof
        .clear();
    Ok(Sha256::digest(canonical.encode_to_vec()).into())
}

pub fn compute_provider_reply_generation_request_digest_v1(
    request: &AiProviderReplyGenerationRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    validate_provider_reply_generation_request_v1(request)?;
    Ok(Sha256::digest(request.encode_to_vec()).into())
}

pub fn seal_summary_inference_request_v1(
    mut request: CommunicationSummaryInferenceRequestV1,
) -> Result<CommunicationSummaryInferenceRequestV1, AiContractValidationErrorV1> {
    let digest = compute_summary_inference_request_digest_v1(&request)?;
    request
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?
        .request_digest = digest.to_vec();
    validate_summary_inference_request_v1(&request)?;
    Ok(request)
}

pub fn compute_summary_inference_request_digest_v1(
    request: &CommunicationSummaryInferenceRequestV1,
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

pub fn compute_provider_summary_generation_request_digest_v1(
    request: &AiProviderSummaryGenerationRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    validate_provider_summary_generation_request_v1(request)?;
    Ok(Sha256::digest(request.encode_to_vec()).into())
}

pub fn validate_reply_inference_request_v1(
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.run_id)
        || !valid_owner(&request.logical_owner_id)
        || !valid_tone(request.tone)
        || !valid_language(request.language)
        || !valid_subject_policy(request.subject_policy)
        || !(1..=AI_MAX_OUTPUT_BYTES_V1).contains(&request.maximum_output_bytes)
        || !(1..=AI_MAX_OUTPUT_TOKENS_V1).contains(&request.maximum_output_tokens)
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
        || context.use_case != AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32
        || !id16(&context.source_evidence_id)
        || context.source_evidence_revision == 0
        || context.contract_major != AI_CONTRACT_MAJOR_V1
        || context.contract_revision != AI_CONTRACT_REVISION_V1
        || context.contract_schema_sha256 != AI_CONTRACTS_SCHEMA_SHA256
        || !sha256(&context.request_digest)
        || context.request_digest != compute_reply_inference_request_digest_v1(request)?
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    let source = request
        .source
        .as_ref()
        .ok_or(AiContractValidationErrorV1::InvalidSource)?;
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

pub fn validate_reply_source_content_v1(
    content: &AiReplySourceContentV1,
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

pub fn encode_reply_source_content_v1(
    content: &AiReplySourceContentV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_reply_source_content_v1(content)?;
    Ok(content.encode_to_vec())
}

pub fn decode_reply_source_content_v1(
    bytes: &[u8],
) -> Result<AiReplySourceContentV1, AiContractValidationErrorV1> {
    let content = AiReplySourceContentV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidSource)?;
    validate_reply_source_content_v1(&content)?;
    Ok(content)
}

pub fn validate_summary_inference_request_v1(
    request: &CommunicationSummaryInferenceRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.run_id)
        || !valid_owner(&request.logical_owner_id)
        || !valid_summary_language(request.language)
        || !valid_summary_length(request.length)
        || !(1..=AI_MAX_OUTPUT_BYTES_V1).contains(&request.maximum_output_bytes)
        || !(1..=AI_MAX_OUTPUT_TOKENS_V1).contains(&request.maximum_output_tokens)
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
        || context.use_case != AiUseCaseV1::AiUseCaseCommunicationSummary as i32
        || !id16(&context.source_evidence_id)
        || context.source_evidence_revision == 0
        || context.contract_major != AI_CONTRACT_MAJOR_V1
        || context.contract_revision != AI_CONTRACT_REVISION_V1
        || context.contract_schema_sha256 != AI_CONTRACTS_SCHEMA_SHA256
        || !sha256(&context.request_digest)
        || context.request_digest != compute_summary_inference_request_digest_v1(request)?
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    validate_private_source(request.source.as_ref())
}

pub fn validate_summary_source_content_v1(
    content: &AiSummarySourceContentV1,
) -> Result<(), AiContractValidationErrorV1> {
    validate_source_fields(
        &content.sender_utf8,
        &content.subject_utf8,
        &content.body_utf8,
        content.encoded_len(),
    )
}

pub fn encode_summary_source_content_v1(
    content: &AiSummarySourceContentV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_summary_source_content_v1(content)?;
    Ok(content.encode_to_vec())
}

pub fn decode_summary_source_content_v1(
    bytes: &[u8],
) -> Result<AiSummarySourceContentV1, AiContractValidationErrorV1> {
    let content = AiSummarySourceContentV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidSource)?;
    validate_summary_source_content_v1(&content)?;
    Ok(content)
}

pub fn validate_summary_inference_result_v1(
    result: &CommunicationSummaryInferenceResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.run_id) || !sha256(&result.request_digest) || !sha256(&result.source_sha256) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if summary_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    if result.summary_utf8.is_empty()
        || result.summary_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.summary_utf8).is_err()
        || !valid_resolved_summary_language(result.resolved_language)
        || !valid_summary_length(result.resolved_length)
        || !valid_completeness(result.completeness)
        || result.confidence_basis_points > 10_000
        || !valid_inference_receipt(result.inference_receipt.as_ref())
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(())
}

pub fn validate_provider_summary_generation_request_v1(
    request: &AiProviderSummaryGenerationRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.request_id)
        || request.input_utf8.is_empty()
        || request.input_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(&request.input_utf8).is_err()
        || !valid_summary_language(request.language)
        || !valid_summary_length(request.length)
        || !(1..=AI_MAX_OUTPUT_BYTES_V1).contains(&request.maximum_output_bytes)
        || !(1..=AI_MAX_OUTPUT_TOKENS_V1).contains(&request.maximum_output_tokens)
        || request.egress_policy != AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32
        || request.egress_policy_revision != AI_LOCAL_EGRESS_POLICY_REVISION_V1
    {
        return Err(AiContractValidationErrorV1::InvalidPolicy);
    }
    Ok(())
}

pub fn validate_provider_summary_generation_result_v1(
    result: &AiProviderSummaryGenerationResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.request_id) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if provider_summary_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    if result.summary_utf8.is_empty()
        || result.summary_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.summary_utf8).is_err()
        || !valid_resolved_summary_language(result.resolved_language)
        || !valid_summary_length(result.resolved_length)
        || !sha256(&result.model_revision_sha256)
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

pub fn validate_reply_inference_result_v1(
    result: &CommunicationReplySuggestionInferenceResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.run_id) || !sha256(&result.request_digest) || !sha256(&result.source_sha256) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let Ok(status) = AiInferenceTerminalStatusV1::try_from(result.terminal_status) else {
        return Err(AiContractValidationErrorV1::InvalidResult);
    };
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if rejected_result_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    if result.subject_utf8.len() > AI_MAX_SUBJECT_BYTES_V1
        || result.body_utf8.is_empty()
        || result.body_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.subject_utf8).is_err()
        || std::str::from_utf8(&result.body_utf8).is_err()
        || !valid_tone(result.resolved_tone)
        || !valid_resolved_language(result.resolved_language)
        || !matches!(
            AiInferenceCompletenessV1::try_from(result.completeness),
            Ok(AiInferenceCompletenessV1::AiInferenceCompletenessComplete
                | AiInferenceCompletenessV1::AiInferenceCompletenessPartial)
        )
        || result.confidence_basis_points > 10_000
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let receipt = result
        .inference_receipt
        .as_ref()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?;
    if !sha256(&receipt.model_revision_sha256)
        || !sha256(&receipt.prompt_policy_sha256)
        || receipt.provider_settings_revision == 0
        || receipt.provider_policy_revision == 0
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    Ok(())
}

pub fn validate_provider_reply_generation_request_v1(
    request: &AiProviderReplyGenerationRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.request_id)
        || request.input_utf8.is_empty()
        || request.input_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(&request.input_utf8).is_err()
        || !valid_tone(request.tone)
        || !valid_language(request.language)
        || !valid_subject_policy(request.subject_policy)
        || !(1..=AI_MAX_OUTPUT_BYTES_V1).contains(&request.maximum_output_bytes)
        || !(1..=AI_MAX_OUTPUT_TOKENS_V1).contains(&request.maximum_output_tokens)
        || request.egress_policy != AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32
        || request.egress_policy_revision != AI_LOCAL_EGRESS_POLICY_REVISION_V1
    {
        return Err(AiContractValidationErrorV1::InvalidPolicy);
    }
    Ok(())
}

pub fn validate_provider_reply_generation_result_v1(
    result: &AiProviderReplyGenerationResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.request_id) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let Ok(status) = AiInferenceTerminalStatusV1::try_from(result.terminal_status) else {
        return Err(AiContractValidationErrorV1::InvalidResult);
    };
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if provider_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    if result.subject_utf8.len() > AI_MAX_SUBJECT_BYTES_V1
        || result.body_utf8.is_empty()
        || result.body_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.subject_utf8).is_err()
        || std::str::from_utf8(&result.body_utf8).is_err()
        || !valid_tone(result.resolved_tone)
        || !valid_resolved_language(result.resolved_language)
        || !sha256(&result.model_revision_sha256)
        || result.output_tokens == 0
        || result.output_tokens > AI_MAX_OUTPUT_TOKENS_V1
        || !matches!(
            AiInferenceCompletenessV1::try_from(result.completeness),
            Ok(AiInferenceCompletenessV1::AiInferenceCompletenessComplete
                | AiInferenceCompletenessV1::AiInferenceCompletenessPartial)
        )
        || result.confidence_basis_points > 10_000
        || result.provider_settings_revision == 0
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(())
}

fn rejected_result_is_sanitized(
    result: &CommunicationReplySuggestionInferenceResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.subject_utf8.is_empty()
        && result.body_utf8.is_empty()
        && result.resolved_tone == AiReplyToneV1::AiReplyToneUnspecified as i32
        && result.resolved_language == AiReplyLanguageV1::AiReplyLanguageUnspecified as i32
        && result.inference_receipt.is_none()
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
}

fn provider_rejection_is_sanitized(
    result: &AiProviderReplyGenerationResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.subject_utf8.is_empty()
        && result.body_utf8.is_empty()
        && result.resolved_tone == AiReplyToneV1::AiReplyToneUnspecified as i32
        && result.resolved_language == AiReplyLanguageV1::AiReplyLanguageUnspecified as i32
        && result.model_revision_sha256.is_empty()
        && result.input_tokens == 0
        && result.output_tokens == 0
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
        && result.provider_settings_revision == 0
}

fn summary_rejection_is_sanitized(
    result: &CommunicationSummaryInferenceResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.summary_utf8.is_empty()
        && result.resolved_language == AiSummaryLanguageV1::AiSummaryLanguageUnspecified as i32
        && result.resolved_length == AiSummaryLengthV1::AiSummaryLengthUnspecified as i32
        && result.inference_receipt.is_none()
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
}

fn provider_summary_rejection_is_sanitized(
    result: &AiProviderSummaryGenerationResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.summary_utf8.is_empty()
        && result.resolved_language == AiSummaryLanguageV1::AiSummaryLanguageUnspecified as i32
        && result.resolved_length == AiSummaryLengthV1::AiSummaryLengthUnspecified as i32
        && result.model_revision_sha256.is_empty()
        && result.input_tokens == 0
        && result.output_tokens == 0
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
        && result.provider_settings_revision == 0
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

fn validate_source_fields(
    sender_utf8: &[u8],
    subject_utf8: &[u8],
    body_utf8: &[u8],
    encoded_len: usize,
) -> Result<(), AiContractValidationErrorV1> {
    if sender_utf8.len() > AI_MAX_SENDER_BYTES_V1
        || subject_utf8.len() > AI_MAX_SUBJECT_BYTES_V1
        || body_utf8.is_empty()
        || body_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || encoded_len > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(sender_utf8).is_err()
        || std::str::from_utf8(subject_utf8).is_err()
        || std::str::from_utf8(body_utf8).is_err()
    {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(())
}

fn valid_inference_receipt(receipt: Option<&AiInferenceReceiptV1>) -> bool {
    receipt.is_some_and(|receipt| {
        sha256(&receipt.model_revision_sha256)
            && sha256(&receipt.prompt_policy_sha256)
            && receipt.provider_settings_revision > 0
            && receipt.provider_policy_revision > 0
    })
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

fn valid_tone(value: i32) -> bool {
    matches!(
        AiReplyToneV1::try_from(value),
        Ok(AiReplyToneV1::AiReplyToneNeutral
            | AiReplyToneV1::AiReplyToneWarm
            | AiReplyToneV1::AiReplyToneFormal
            | AiReplyToneV1::AiReplyToneConcise)
    )
}

fn valid_language(value: i32) -> bool {
    matches!(
        AiReplyLanguageV1::try_from(value),
        Ok(AiReplyLanguageV1::AiReplyLanguageAuto
            | AiReplyLanguageV1::AiReplyLanguageEnglish
            | AiReplyLanguageV1::AiReplyLanguageSpanish
            | AiReplyLanguageV1::AiReplyLanguageRussian)
    )
}

fn valid_resolved_language(value: i32) -> bool {
    matches!(
        AiReplyLanguageV1::try_from(value),
        Ok(AiReplyLanguageV1::AiReplyLanguageEnglish
            | AiReplyLanguageV1::AiReplyLanguageSpanish
            | AiReplyLanguageV1::AiReplyLanguageRussian)
    )
}

fn valid_subject_policy(value: i32) -> bool {
    matches!(
        AiReplySubjectPolicyV1::try_from(value),
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve
            | AiReplySubjectPolicyV1::AiReplySubjectPolicyGenerateIfMissing
            | AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit)
    )
}

fn valid_summary_language(value: i32) -> bool {
    matches!(
        AiSummaryLanguageV1::try_from(value),
        Ok(AiSummaryLanguageV1::AiSummaryLanguageAuto
            | AiSummaryLanguageV1::AiSummaryLanguageEnglish
            | AiSummaryLanguageV1::AiSummaryLanguageSpanish
            | AiSummaryLanguageV1::AiSummaryLanguageRussian)
    )
}

fn valid_resolved_summary_language(value: i32) -> bool {
    matches!(
        AiSummaryLanguageV1::try_from(value),
        Ok(AiSummaryLanguageV1::AiSummaryLanguageEnglish
            | AiSummaryLanguageV1::AiSummaryLanguageSpanish
            | AiSummaryLanguageV1::AiSummaryLanguageRussian)
    )
}

fn valid_summary_length(value: i32) -> bool {
    matches!(
        AiSummaryLengthV1::try_from(value),
        Ok(AiSummaryLengthV1::AiSummaryLengthShort
            | AiSummaryLengthV1::AiSummaryLengthStandard
            | AiSummaryLengthV1::AiSummaryLengthDetailed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        AiContextReceiptV1, AiPrivateSourceReceiptV1, AiProviderSummaryGenerationRequestV1,
        AiProviderSummaryGenerationResultV1, AiSummaryLanguageV1, AiSummaryLengthV1,
        AiSummarySourceContentV1, CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1, CommunicationSummaryInferenceRequestV1,
    };

    fn request() -> CommunicationReplySuggestionInferenceRequestV1 {
        CommunicationReplySuggestionInferenceRequestV1 {
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
            tone: AiReplyToneV1::AiReplyToneNeutral as i32,
            language: AiReplyLanguageV1::AiReplyLanguageAuto as i32,
            subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
            maximum_output_bytes: 4_096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn request_digest_is_deterministic_and_context_bound() {
        let request = seal_reply_inference_request_v1(request()).expect("request");
        validate_reply_inference_request_v1(&request).expect("valid");
        let mut renewed = request.clone();
        renewed
            .source
            .as_mut()
            .expect("source")
            .custody_transfer_source_proof = vec![9; 80];
        renewed = seal_reply_inference_request_v1(renewed).expect("renewed request");
        assert_eq!(
            request.context.as_ref().expect("context").request_digest,
            renewed.context.as_ref().expect("context").request_digest
        );
        let mut changed = request.clone();
        changed.maximum_output_tokens += 1;
        assert_eq!(
            validate_reply_inference_request_v1(&changed),
            Err(AiContractValidationErrorV1::InvalidReceipt)
        );
    }

    #[test]
    fn reply_source_content_is_utf8_and_bounded_as_one_blob() {
        let mut content = AiReplySourceContentV1 {
            sender_utf8: b"sender@example.test".to_vec(),
            subject_utf8: b"Subject".to_vec(),
            body_utf8: b"Private source body".to_vec(),
        };
        validate_reply_source_content_v1(&content).expect("source content");

        content.body_utf8 = vec![b'a'; AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize];
        assert_eq!(
            validate_reply_source_content_v1(&content),
            Err(AiContractValidationErrorV1::InvalidSource)
        );

        content.body_utf8 = vec![0xff];
        assert_eq!(
            validate_reply_source_content_v1(&content),
            Err(AiContractValidationErrorV1::InvalidSource)
        );
    }

    #[test]
    fn provider_request_has_no_caller_selected_identity_or_remote_egress() {
        let request = AiProviderReplyGenerationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"private source".to_vec(),
            tone: AiReplyToneV1::AiReplyToneWarm as i32,
            language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
            subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit as i32,
            maximum_output_bytes: 1_024,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        validate_provider_reply_generation_request_v1(&request).expect("provider request");
    }

    #[test]
    fn provider_result_carries_validated_completeness_and_confidence() {
        let mut result = AiProviderReplyGenerationResultV1 {
            request_id: vec![1; 16],
            subject_utf8: b"Re: subject".to_vec(),
            body_utf8: b"Suggested reply".to_vec(),
            resolved_tone: AiReplyToneV1::AiReplyToneWarm as i32,
            resolved_language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
            model_revision_sha256: vec![2; 32],
            input_tokens: 12,
            output_tokens: 8,
            terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
            confidence_basis_points: 8_000,
            provider_settings_revision: 11,
        };
        validate_provider_reply_generation_result_v1(&result).expect("provider result");
        result.confidence_basis_points = 10_001;
        assert_eq!(
            validate_provider_reply_generation_result_v1(&result),
            Err(AiContractValidationErrorV1::InvalidResult)
        );
    }

    #[test]
    fn terminal_rejection_carries_no_candidate_or_provider_receipt() {
        let result = CommunicationReplySuggestionInferenceResultV1 {
            run_id: vec![1; 16],
            request_digest: vec![2; 32],
            source_sha256: vec![3; 32],
            subject_utf8: Vec::new(),
            body_utf8: Vec::new(),
            resolved_tone: AiReplyToneV1::AiReplyToneUnspecified as i32,
            resolved_language: AiReplyLanguageV1::AiReplyLanguageUnspecified as i32,
            inference_receipt: None,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32,
            confidence_basis_points: 0,
            terminal_status:
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32,
        };
        validate_reply_inference_result_v1(&result).expect("sanitized rejection");
    }

    fn summary_request() -> CommunicationSummaryInferenceRequestV1 {
        CommunicationSummaryInferenceRequestV1 {
            run_id: vec![11; 16],
            context: Some(AiContextReceiptV1 {
                context_id: vec![12; 16],
                use_case: AiUseCaseV1::AiUseCaseCommunicationSummary as i32,
                source_evidence_id: vec![13; 16],
                source_evidence_revision: 2,
                contract_major: AI_CONTRACT_MAJOR_V1,
                contract_revision: AI_CONTRACT_REVISION_V1,
                contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                request_digest: Vec::new(),
            }),
            source: Some(AiPrivateSourceReceiptV1 {
                reference_id: vec![14; 16],
                declared_bytes: 128,
                sha256: vec![15; 32],
                custody_transfer_source_proof: vec![16; 64],
            }),
            language: AiSummaryLanguageV1::AiSummaryLanguageAuto as i32,
            length: AiSummaryLengthV1::AiSummaryLengthStandard as i32,
            maximum_output_bytes: 4_096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn summary_request_is_a_distinct_sealed_use_case() {
        let summary = seal_summary_inference_request_v1(summary_request()).expect("summary");
        validate_summary_inference_request_v1(&summary).expect("valid");
        assert_ne!(
            summary.context.expect("context").request_digest,
            seal_reply_inference_request_v1(request())
                .expect("reply")
                .context
                .expect("context")
                .request_digest
        );
    }

    #[test]
    fn summary_source_and_provider_contracts_are_bounded() {
        let source = AiSummarySourceContentV1 {
            sender_utf8: b"sender@example.test".to_vec(),
            subject_utf8: b"Subject".to_vec(),
            body_utf8: b"Private source".to_vec(),
        };
        let encoded = encode_summary_source_content_v1(&source).expect("source");
        assert_eq!(decode_summary_source_content_v1(&encoded), Ok(source));

        let request = AiProviderSummaryGenerationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"private source".to_vec(),
            language: AiSummaryLanguageV1::AiSummaryLanguageRussian as i32,
            length: AiSummaryLengthV1::AiSummaryLengthShort as i32,
            maximum_output_bytes: 2_048,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        validate_provider_summary_generation_request_v1(&request).expect("provider request");
        assert_ne!(
            compute_provider_summary_generation_request_digest_v1(&request).expect("digest"),
            [0; 32]
        );

        let result = AiProviderSummaryGenerationResultV1 {
            request_id: vec![1; 16],
            summary_utf8: b"Bounded summary".to_vec(),
            resolved_language: AiSummaryLanguageV1::AiSummaryLanguageRussian as i32,
            resolved_length: AiSummaryLengthV1::AiSummaryLengthShort as i32,
            model_revision_sha256: vec![2; 32],
            input_tokens: 10,
            output_tokens: 4,
            terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
            confidence_basis_points: 7_500,
            provider_settings_revision: 3,
        };
        validate_provider_summary_generation_result_v1(&result).expect("provider result");
    }
}
