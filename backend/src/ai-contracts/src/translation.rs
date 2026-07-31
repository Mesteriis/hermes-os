use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_CUSTODY_PROOF_BYTES_V1, AI_MAX_OUTPUT_BYTES_V1,
    AI_MAX_OUTPUT_TOKENS_V1, AI_MAX_PRIVATE_SOURCE_BYTES_V1, AI_MAX_SENDER_BYTES_V1,
    AI_MAX_SUBJECT_BYTES_V1,
    validation::AiContractValidationErrorV1,
    wire::{
        AiDetectedLanguageV1, AiEgressPolicyV1, AiInferenceCompletenessV1, AiInferenceReceiptV1,
        AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1, AiProviderTranslationRequestV1,
        AiProviderTranslationResultV1, AiTranslationLanguageV1, AiTranslationSourceContentV1,
        AiUseCaseV1, CommunicationTranslationInferenceRequestV1,
        CommunicationTranslationInferenceResultV1,
    },
};

pub fn seal_translation_inference_request_v1(
    mut request: CommunicationTranslationInferenceRequestV1,
) -> Result<CommunicationTranslationInferenceRequestV1, AiContractValidationErrorV1> {
    let digest = compute_translation_inference_request_digest_v1(&request)?;
    request
        .context
        .as_mut()
        .ok_or(AiContractValidationErrorV1::InvalidReceipt)?
        .request_digest = digest.to_vec();
    validate_translation_inference_request_v1(&request)?;
    Ok(request)
}

pub fn compute_translation_inference_request_digest_v1(
    request: &CommunicationTranslationInferenceRequestV1,
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

pub fn compute_provider_translation_request_digest_v1(
    request: &AiProviderTranslationRequestV1,
) -> Result<[u8; 32], AiContractValidationErrorV1> {
    validate_provider_translation_request_v1(request)?;
    Ok(Sha256::digest(request.encode_to_vec()).into())
}

pub fn validate_translation_inference_request_v1(
    request: &CommunicationTranslationInferenceRequestV1,
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
        || context.use_case != AiUseCaseV1::AiUseCaseCommunicationTranslation as i32
        || !id16(&context.source_evidence_id)
        || context.source_evidence_revision == 0
        || context.contract_major != AI_CONTRACT_MAJOR_V1
        || context.contract_revision != AI_CONTRACT_REVISION_V1
        || context.contract_schema_sha256 != AI_CONTRACTS_SCHEMA_SHA256
        || !sha256(&context.request_digest)
        || context.request_digest != compute_translation_inference_request_digest_v1(request)?
    {
        return Err(AiContractValidationErrorV1::InvalidReceipt);
    }
    validate_private_source(request.source.as_ref())
}

pub fn validate_translation_source_content_v1(
    content: &AiTranslationSourceContentV1,
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

pub fn encode_translation_source_content_v1(
    content: &AiTranslationSourceContentV1,
) -> Result<Vec<u8>, AiContractValidationErrorV1> {
    validate_translation_source_content_v1(content)?;
    Ok(content.encode_to_vec())
}

pub fn decode_translation_source_content_v1(
    bytes: &[u8],
) -> Result<AiTranslationSourceContentV1, AiContractValidationErrorV1> {
    let content = AiTranslationSourceContentV1::decode(bytes)
        .map_err(|_| AiContractValidationErrorV1::InvalidSource)?;
    validate_translation_source_content_v1(&content)?;
    Ok(content)
}

pub fn validate_translation_inference_result_v1(
    result: &CommunicationTranslationInferenceResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.run_id) || !sha256(&result.request_digest) || !sha256(&result.source_sha256) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if translation_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
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

pub fn validate_provider_translation_request_v1(
    request: &AiProviderTranslationRequestV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&request.request_id)
        || request.input_utf8.is_empty()
        || request.input_utf8.len() > AI_MAX_PRIVATE_SOURCE_BYTES_V1 as usize
        || std::str::from_utf8(&request.input_utf8).is_err()
        || !valid_translation_language(request.target_language)
        || !(1..=AI_MAX_OUTPUT_BYTES_V1).contains(&request.maximum_output_bytes)
        || !(1..=AI_MAX_OUTPUT_TOKENS_V1).contains(&request.maximum_output_tokens)
        || request.egress_policy != AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32
        || request.egress_policy_revision != AI_LOCAL_EGRESS_POLICY_REVISION_V1
    {
        return Err(AiContractValidationErrorV1::InvalidPolicy);
    }
    Ok(())
}

pub fn validate_provider_translation_result_v1(
    result: &AiProviderTranslationResultV1,
) -> Result<(), AiContractValidationErrorV1> {
    if !id16(&result.request_id) {
        return Err(AiContractValidationErrorV1::InvalidResult);
    }
    let status = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AiContractValidationErrorV1::InvalidResult)?;
    if status != AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady {
        return if provider_translation_rejection_is_sanitized(result, status) {
            Ok(())
        } else {
            Err(AiContractValidationErrorV1::InvalidResult)
        };
    }
    if result.translated_text_utf8.is_empty()
        || result.translated_text_utf8.len() > AI_MAX_OUTPUT_BYTES_V1 as usize
        || std::str::from_utf8(&result.translated_text_utf8).is_err()
        || !valid_detected_language(result.detected_source_language)
        || !valid_translation_language(result.target_language)
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

fn translation_rejection_is_sanitized(
    result: &CommunicationTranslationInferenceResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.translated_text_utf8.is_empty()
        && result.detected_source_language
            == AiDetectedLanguageV1::AiDetectedLanguageUnspecified as i32
        && result.target_language
            == AiTranslationLanguageV1::AiTranslationLanguageUnspecified as i32
        && result.inference_receipt.is_none()
        && result.completeness
            == AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32
        && result.confidence_basis_points == 0
}

fn provider_translation_rejection_is_sanitized(
    result: &AiProviderTranslationResultV1,
    status: AiInferenceTerminalStatusV1,
) -> bool {
    rejection_status(status)
        && result.translated_text_utf8.is_empty()
        && result.detected_source_language
            == AiDetectedLanguageV1::AiDetectedLanguageUnspecified as i32
        && result.target_language
            == AiTranslationLanguageV1::AiTranslationLanguageUnspecified as i32
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

fn valid_translation_language(value: i32) -> bool {
    matches!(
        AiTranslationLanguageV1::try_from(value),
        Ok(AiTranslationLanguageV1::AiTranslationLanguageEnglish
            | AiTranslationLanguageV1::AiTranslationLanguageSpanish
            | AiTranslationLanguageV1::AiTranslationLanguageRussian)
    )
}

fn valid_detected_language(value: i32) -> bool {
    matches!(
        AiDetectedLanguageV1::try_from(value),
        Ok(AiDetectedLanguageV1::AiDetectedLanguageUnknown
            | AiDetectedLanguageV1::AiDetectedLanguageEnglish
            | AiDetectedLanguageV1::AiDetectedLanguageSpanish
            | AiDetectedLanguageV1::AiDetectedLanguageRussian)
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

    #[test]
    fn translation_is_a_distinct_sealed_use_case_with_exact_target_language() {
        let request = CommunicationTranslationInferenceRequestV1 {
            run_id: vec![21; 16],
            context: Some(AiContextReceiptV1 {
                context_id: vec![22; 16],
                use_case: AiUseCaseV1::AiUseCaseCommunicationTranslation as i32,
                source_evidence_id: vec![23; 16],
                source_evidence_revision: 5,
                contract_major: AI_CONTRACT_MAJOR_V1,
                contract_revision: AI_CONTRACT_REVISION_V1,
                contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                request_digest: Vec::new(),
            }),
            source: Some(AiPrivateSourceReceiptV1 {
                reference_id: vec![24; 16],
                declared_bytes: 128,
                sha256: vec![25; 32],
                custody_transfer_source_proof: vec![26; 64],
            }),
            target_language: AiTranslationLanguageV1::AiTranslationLanguageRussian as i32,
            maximum_output_bytes: 4_096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
            logical_owner_id: "owner-1".to_owned(),
        };
        let sealed = seal_translation_inference_request_v1(request).expect("translation");
        validate_translation_inference_request_v1(&sealed).expect("valid");
        let mut unsupported = sealed;
        unsupported.target_language =
            AiTranslationLanguageV1::AiTranslationLanguageUnspecified as i32;
        assert_eq!(
            validate_translation_inference_request_v1(&unsupported),
            Err(AiContractValidationErrorV1::InvalidRequest)
        );
    }

    #[test]
    fn translation_source_and_provider_contracts_are_bounded_and_provider_neutral() {
        let source = AiTranslationSourceContentV1 {
            sender_utf8: b"sender@example.test".to_vec(),
            subject_utf8: b"Subject".to_vec(),
            body_utf8: b"Translate private source".to_vec(),
        };
        let encoded = encode_translation_source_content_v1(&source).expect("source");
        assert_eq!(decode_translation_source_content_v1(&encoded), Ok(source));

        let request = AiProviderTranslationRequestV1 {
            request_id: vec![31; 16],
            input_utf8: b"private source".to_vec(),
            target_language: AiTranslationLanguageV1::AiTranslationLanguageSpanish as i32,
            maximum_output_bytes: 2_048,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        validate_provider_translation_request_v1(&request).expect("provider request");
        assert_ne!(
            compute_provider_translation_request_digest_v1(&request).expect("digest"),
            [0; 32]
        );
        let result = AiProviderTranslationResultV1 {
            request_id: vec![31; 16],
            translated_text_utf8: b"fuente privada".to_vec(),
            detected_source_language: AiDetectedLanguageV1::AiDetectedLanguageEnglish as i32,
            target_language: AiTranslationLanguageV1::AiTranslationLanguageSpanish as i32,
            model_revision_sha256: vec![32; 32],
            input_tokens: 10,
            output_tokens: 4,
            terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
            completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
            confidence_basis_points: 8_000,
            provider_settings_revision: 3,
        };
        validate_provider_translation_result_v1(&result).expect("provider result");
    }
}
