use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_CUSTODY_PROOF_BYTES_V1, AI_MAX_OUTPUT_BYTES_V1,
    AI_MAX_OUTPUT_TOKENS_V1,
    validation::AiContractValidationErrorV1,
    wire::{
        AiDetectedLanguageV1, AiEgressPolicyV1, AiInferenceCompletenessV1,
        AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1, AiTranslationLanguageV1,
        AiUseCaseV1, AttachmentTranslationInferenceRequestV1,
        AttachmentTranslationInferenceResultV1,
    },
};

pub const AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1: u64 = 1024 * 1024;

pub fn seal_attachment_translation_inference_request_v1(
    mut request: AttachmentTranslationInferenceRequestV1,
) -> Result<AttachmentTranslationInferenceRequestV1, AiContractValidationErrorV1> {
    let digest = compute_attachment_translation_inference_request_digest_v1(&request)?;
    request
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?
        .request_digest = digest.to_vec();
    validate_attachment_translation_inference_request_v1(&request)?;
    Ok(request)
}

pub fn compute_attachment_translation_inference_request_digest_v1(
    request: &AttachmentTranslationInferenceRequestV1,
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

pub fn validate_attachment_translation_inference_request_v1(
    request: &AttachmentTranslationInferenceRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.run_id)
        || !valid_owner(&request.logical_owner_id)
        || !valid_translation_language(request.target_language)
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
        || context.use_case != AiUseCaseV1::AiUseCaseAttachmentTranslation as i32
        || !id16(&context.source_evidence_id)
        || context.source_evidence_revision == 0
        || context.contract_major != AI_CONTRACT_MAJOR_V1
        || context.contract_revision != AI_CONTRACT_REVISION_V1
        || context.contract_schema_sha256 != AI_CONTRACTS_SCHEMA_SHA256
        || !sha256(&context.request_digest)
        || context.request_digest
            != compute_attachment_translation_inference_request_digest_v1(request)?
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    validate_private_source(request.source.as_ref())
}

pub fn validate_attachment_translation_source_text_v1(
    source: &AiPrivateSourceReceiptV1,
    source_text_utf8: &[u8],
) -> Result<(), AiContractValidationErrorV1> {
    validate_private_source(Some(source))?;
    if source_text_utf8.is_empty()
        || u64::try_from(source_text_utf8.len()).ok() != Some(source.declared_bytes)
        || std::str::from_utf8(source_text_utf8).is_err()
        || Sha256::digest(source_text_utf8).as_slice() != source.sha256
    {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(())
}

pub fn validate_attachment_translation_inference_result_v1(
    result: &AttachmentTranslationInferenceResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.run_id) || !sha256(&result.request_digest) || !sha256(&result.source_sha256) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return sanitized_rejection(result, status)
            .then_some(())
            .ok_or(AiContractValidationErrorV1::InvalidResult);
    }
    if result.translated_text_utf8.is_empty()
        || result.translated_text_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.translated_text_utf8).is_err()
        || !valid_detected_language(result.detected_source_language)
        || !valid_translation_language(result.target_language)
        || !valid_completeness(result.completeness)
        || result.confidence_basis_points > 10_000
        || !valid_inference_receipt(result.inference_receipt.as_ref())
    {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    Ok(())
}

fn validate_private_source(
    source: Option<&AiPrivateSourceReceiptV1>,
) -> Result<(), AiContractValidationErrorV1> {
    let source = source.ok_or(AiContractValidationErrorV1::InvalidSource)?;
    if !id16(&source.reference_id)
        || !(1..=AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&source.declared_bytes)
        || !sha256(&source.sha256)
        || source.custody_transfer_source_proof.is_empty()
        || source.custody_transfer_source_proof.len() > AI_MAX_CUSTODY_PROOF_BYTES_V1
    {
        return Err(AiContractValidationErrorV1::InvalidSource);
    }
    Ok(())
}

fn sanitized_rejection(
    result: &AttachmentTranslationInferenceResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    matches!(
        status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected
    ) && result.translated_text_utf8.is_empty()
        && result.detected_source_language == 0
        && result.target_language == 0
        && result.inference_receipt.is_none()
        && result.completeness == 0
        && result.confidence_basis_points == 0
}

fn valid_inference_receipt(receipt: Option<&crate::wire::AiInferenceReceiptV1>) -> bool {
    receipt.is_some_and(|receipt| {
        sha256(&receipt.model_revision_sha256)
            && sha256(&receipt.prompt_policy_sha256)
            && receipt.provider_settings_revision > 0
            && receipt.provider_policy_revision > 0
    })
}

fn valid_translation_language(value: i32) -> bool {
    matches!(
        AiTranslationLanguageV1::try_from(value),
        Ok(AiTranslationLanguageV1::AiTranslationLanguageEnglish)
            | Ok(AiTranslationLanguageV1::AiTranslationLanguageSpanish)
            | Ok(AiTranslationLanguageV1::AiTranslationLanguageRussian)
    )
}

fn valid_detected_language(value: i32) -> bool {
    matches!(
        AiDetectedLanguageV1::try_from(value),
        Ok(AiDetectedLanguageV1::AiDetectedLanguageUnknown)
            | Ok(AiDetectedLanguageV1::AiDetectedLanguageEnglish)
            | Ok(AiDetectedLanguageV1::AiDetectedLanguageSpanish)
            | Ok(AiDetectedLanguageV1::AiDetectedLanguageRussian)
    )
}

fn valid_completeness(value: i32) -> bool {
    matches!(
        AiInferenceCompletenessV1::try_from(value),
        Ok(AiInferenceCompletenessV1::AiInferenceCompletenessComplete)
            | Ok(AiInferenceCompletenessV1::AiInferenceCompletenessPartial)
    )
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
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
    use crate::wire::{AiContextReceiptV1, AiEgressPolicyV1};

    fn request() -> AttachmentTranslationInferenceRequestV1 {
        AttachmentTranslationInferenceRequestV1 {
            run_id: vec![1; 16],
            context: Some(AiContextReceiptV1 {
                context_id: vec![2; 16],
                use_case: AiUseCaseV1::AiUseCaseAttachmentTranslation as i32,
                source_evidence_id: vec![3; 16],
                source_evidence_revision: 7,
                contract_major: AI_CONTRACT_MAJOR_V1,
                contract_revision: AI_CONTRACT_REVISION_V1,
                contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                request_digest: Vec::new(),
            }),
            source: Some(AiPrivateSourceReceiptV1 {
                reference_id: vec![4; 16],
                declared_bytes: 4,
                sha256: Sha256::digest(b"text").to_vec(),
                custody_transfer_source_proof: vec![5; 64],
            }),
            target_language: AiTranslationLanguageV1::AiTranslationLanguageRussian as i32,
            maximum_output_bytes: 4096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn seals_distinct_attachment_use_case_and_binds_raw_text_receipt() {
        let request = seal_attachment_translation_inference_request_v1(request()).expect("seal");
        validate_attachment_translation_inference_request_v1(&request).expect("valid");
        validate_attachment_translation_source_text_v1(
            request.source.as_ref().expect("source"),
            b"text",
        )
        .expect("source");
    }

    #[test]
    fn rejects_communication_use_case_and_digest_drift() {
        let mut wrong_use_case = request();
        wrong_use_case.context.as_mut().expect("context").use_case =
            AiUseCaseV1::AiUseCaseCommunicationTranslation as i32;
        assert_eq!(
            seal_attachment_translation_inference_request_v1(wrong_use_case),
            Err(AiContractValidationErrorV1::InvalidReceipt)
        );
        let request = seal_attachment_translation_inference_request_v1(request()).expect("seal");
        assert_eq!(
            validate_attachment_translation_source_text_v1(
                request.source.as_ref().expect("source"),
                b"drift",
            ),
            Err(AiContractValidationErrorV1::InvalidSource)
        );
    }
}
