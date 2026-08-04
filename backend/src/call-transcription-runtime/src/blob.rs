use std::os::unix::net::UnixStream;

use hermes_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use hermes_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use hermes_speech_to_text_api::{
    SPEECH_TO_TEXT_CAPABILITY_ID_V1, SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1,
};
use sha2::{Digest, Sha256};

use crate::admission::BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordingCustodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranscriptCustodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionBlobErrorV1 {
    InvalidReceipt,
    Rejected,
    Unavailable,
}

#[allow(clippy::too_many_arguments)]
pub fn accept_recording_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    custody_source_proof: &[u8],
    event_id: [u8; 16],
    envelope_sha256: [u8; 32],
) -> Result<RecordingCustodyReceiptV1, CallTranscriptionBlobErrorV1> {
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source_reference_id,
            declared_size: declared_bytes,
            receipt_sha256: &receipt_sha256,
            custody_source_proof,
            evidence_id: &event_id,
            evidence_envelope_sha256: &envelope_sha256,
        },
    )
    .map_err(classify)?;
    let target_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(classify)?;
    Ok(RecordingCustodyReceiptV1 {
        reference_id: target_reference_id,
        declared_bytes,
        receipt_sha256,
    })
}

pub fn fresh_stt_source_proof_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &RecordingCustodyReceiptV1,
) -> Result<Vec<u8>, CallTranscriptionBlobErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&source.receipt_sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: SPEECH_TO_TEXT_OWNER_V1,
                module_id: SPEECH_TO_TEXT_MODULE_ID_V1,
                capability_id: SPEECH_TO_TEXT_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(classify)?;
    if session.custody_transfer_source_proof.is_empty()
        || session.custody_transfer_source_proof.len() > 2_048
    {
        return Err(CallTranscriptionBlobErrorV1::InvalidReceipt);
    }
    Ok(session.custody_transfer_source_proof)
}

pub fn verify_transcript_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    transcript: &TranscriptCustodyReceiptV1,
) -> Result<(), CallTranscriptionBlobErrorV1> {
    request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &transcript.reference_id,
            declared_size: transcript.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&transcript.receipt_sha256),
            custody_target: None,
        },
    )
    .map_err(classify)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn accept_transcript_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    custody_source_proof: &[u8],
    request_id: [u8; 16],
    result_receipt_sha256: [u8; 32],
) -> Result<TranscriptCustodyReceiptV1, CallTranscriptionBlobErrorV1> {
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source_reference_id,
            declared_size: declared_bytes,
            receipt_sha256: &receipt_sha256,
            custody_source_proof,
            evidence_id: &request_id,
            evidence_envelope_sha256: &result_receipt_sha256,
        },
    )
    .map_err(classify)?;
    let target_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(classify)?;
    Ok(TranscriptCustodyReceiptV1 {
        reference_id: target_reference_id,
        declared_bytes,
        receipt_sha256,
    })
}

pub fn release_recording_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source: &RecordingCustodyReceiptV1,
    fresh_source_proof: &[u8],
    accepted: bool,
) -> Result<(), CallTranscriptionBlobErrorV1> {
    let operation_id = release_operation_id(run_id, source.receipt_sha256);
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &operation_id,
            capability_id: BLOB_CAPABILITY_ID_V1,
            reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.receipt_sha256,
            custody_source_proof: fresh_source_proof,
            reason: if accepted {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            } else {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            },
        },
    )
    .map_err(classify)?;
    Ok(())
}

fn release_operation_id(run_id: [u8; 16], receipt_sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"hermes.call-transcription.release-recording.v1\0");
    digest.update(run_id);
    digest.update(receipt_sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionBlobErrorV1::InvalidReceipt)
}

fn classify(error: BlobClientError) -> CallTranscriptionBlobErrorV1 {
    match error {
        BlobClientError::Unavailable => CallTranscriptionBlobErrorV1::Unavailable,
        BlobClientError::Rejected(_) => CallTranscriptionBlobErrorV1::Rejected,
        _ => CallTranscriptionBlobErrorV1::InvalidReceipt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_identity_is_content_and_run_bound() {
        assert_eq!(
            release_operation_id([1; 16], [2; 32]),
            release_operation_id([1; 16], [2; 32])
        );
        assert_ne!(
            release_operation_id([1; 16], [2; 32]),
            release_operation_id([1; 16], [3; 32])
        );
    }
}
