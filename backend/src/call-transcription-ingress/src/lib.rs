#![forbid(unsafe_code)]

use hermes_runtime_protocol::v1::ContractReferenceV1;

pub const PACKAGE: &str = "hermes-call-transcription-ingress";
pub const OWNER_ID_V1: &str = "call_transcription";
pub const RECORDING_READY_CONTRACT_NAME_V1: &str = "call_transcription.recording_ready";
pub const RECORDING_REJECTED_CONTRACT_NAME_V1: &str = "call_transcription.recording_rejected";
pub const CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTRACT_REVISION_V1: u32 = 1;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.call_transcription.ingress.v1.rs"
    ));
}
include!(concat!(
    env!("OUT_DIR"),
    "/call_transcription_ingress_schema.rs"
));
pub const DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/call-transcription-ingress-v1.bin"
));

#[must_use]
pub fn contract_reference_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CONTRACT_MAJOR_V1,
        revision: CONTRACT_REVISION_V1,
        schema_sha256: CALL_TRANSCRIPTION_INGRESS_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn event_is_target_owned_and_has_no_audio_or_path() {
        let source = include_str!("../proto/hermes/call_transcription/ingress/v1/recording.proto");
        for required in [
            "consent_receipt_id",
            "target_blob_reference_id",
            "custody_transfer_source_proof",
            "logical_owner_id",
        ] {
            assert!(source.contains(required));
        }
        for forbidden in ["audio_bytes", "filesystem_path", "provider_id", "device_id"] {
            assert!(!source.contains(forbidden));
        }
    }
}
