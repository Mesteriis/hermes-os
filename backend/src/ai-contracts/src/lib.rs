#![forbid(unsafe_code)]

mod validation;

use hermes_runtime_protocol::v1::ContractReferenceV1;
pub use validation::{
    AiContractValidationErrorV1, compute_provider_reply_generation_request_digest_v1,
    compute_reply_inference_request_digest_v1, decode_reply_source_content_v1,
    encode_reply_source_content_v1, seal_reply_inference_request_v1,
    validate_provider_reply_generation_request_v1, validate_provider_reply_generation_result_v1,
    validate_reply_inference_request_v1, validate_reply_inference_result_v1,
    validate_reply_source_content_v1,
};

pub const PACKAGE: &str = "hermes-ai-contracts";
pub const AI_OWNER_V1: &str = "ai";
pub const AI_CONTRACT_MAJOR_V1: u32 = 1;
pub const AI_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_REPLY_INFERENCE_CONTRACT_NAME_V1: &str =
    "communication_reply_suggestion_inference";
pub const AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1: &str = "ai_provider_reply_generation";
pub const AI_INFERENCE_REQUEST_CAPABILITY_ID_V1: &str = "ai.inference.request.v1";
pub const AI_PROVIDER_GENERATION_CAPABILITY_ID_V1: &str = "ai.provider.generate.v1";
pub const AI_INFERENCE_BLOB_CAPABILITY_ID_V1: &str = "ai.inference.blob.v1";
pub const AI_INFERENCE_MODULE_ID_V1: &str = "hermes-ai-inference-runtime";
pub const AI_MAX_PRIVATE_SOURCE_BYTES_V1: u64 = 256 * 1024;
pub const AI_MAX_OUTPUT_BYTES_V1: u32 = 64 * 1024;
pub const AI_MAX_OUTPUT_TOKENS_V1: u32 = 4_096;
pub const AI_MAX_SENDER_BYTES_V1: usize = 512;
pub const AI_MAX_SUBJECT_BYTES_V1: usize = 998;
pub const AI_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const AI_LOCAL_EGRESS_POLICY_REVISION_V1: u32 = 1;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.ai.contracts.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/ai_contracts_schema.rs"));

pub const AI_CONTRACTS_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ai-contracts-v1.bin"));

#[must_use]
pub fn communication_reply_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_REPLY_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn ai_provider_reply_generation_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1)
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: AI_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: AI_CONTRACT_MAJOR_V1,
        revision: AI_CONTRACT_REVISION_V1,
        schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::v1::ContractReferenceV1;

    use super::*;

    #[test]
    fn contracts_are_exact_and_ai_owned() {
        assert_eq!(
            communication_reply_inference_contract_reference_v1(),
            ContractReferenceV1 {
                owner: "ai".to_owned(),
                name: "communication_reply_suggestion_inference".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            }
        );
        assert_eq!(
            ai_provider_reply_generation_contract_reference_v1().owner,
            "ai"
        );
    }
}
