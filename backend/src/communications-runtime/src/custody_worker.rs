//! Executes an owner-local Blob custody transfer for one admitted body.

use std::os::unix::net::UnixStream;

use hermes_blob_client::{
    request_managed_blob_custody_transfer, BlobClientError, BlobDataClient,
};
use hermes_communications_api::CommunicationBodyBlobReferenceV1;
use hermes_communications_persistence::{
    CommunicationsBodyCustodyTransferErrorV1, CommunicationsDurablePersistence,
};

use crate::admission::COMMUNICATIONS_BLOB_CAPABILITY_ID;

const CUSTODY_TRANSFER_LEASE_SECONDS: i64 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsCustodyWorkerErrorV1 {
    StorageUnavailable,
    RetryPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlobCustodyTransferFailureV1 {
    PolicyRejected,
    RetryPending,
}

pub async fn process_next_body_custody_transfer_v1(
    control_channel: &mut UnixStream,
    persistence: &CommunicationsDurablePersistence,
    worker_id: &str,
    now_unix_seconds: i64,
) -> Result<bool, CommunicationsCustodyWorkerErrorV1> {
    let lease_expires_at_unix_seconds = now_unix_seconds
        .checked_add(CUSTODY_TRANSFER_LEASE_SECONDS)
        .ok_or(CommunicationsCustodyWorkerErrorV1::StorageUnavailable)?;
    let Some(claimed) = persistence
        .claim_next_body_custody_transfer(
            worker_id,
            now_unix_seconds,
            lease_expires_at_unix_seconds,
        )
        .await
        .map_err(storage_error)?
    else {
        return Ok(false);
    };

    control_channel
        .set_nonblocking(false)
        .map_err(|_| CommunicationsCustodyWorkerErrorV1::StorageUnavailable)?;
    let transfer = (|| {
        let session = request_managed_blob_custody_transfer(
            control_channel,
            COMMUNICATIONS_BLOB_CAPABILITY_ID,
            &claimed.source_reference_id,
            claimed.declared_bytes,
            &claimed.plaintext_sha256,
            &claimed.source_custody_proof,
            &claimed.evidence_id.bytes(),
            &claimed.envelope_sha256,
        )?;
        let target_reference_id = session
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| BlobClientError::InvalidResponse)?;
        BlobDataClient::new(&session.data_socket_path)?.custody_transfer(
            session.grant,
            session.channel_binding,
        )?;
        Ok::<[u8; 16], BlobClientError>(target_reference_id)
    })();
    control_channel
        .set_nonblocking(true)
        .map_err(|_| CommunicationsCustodyWorkerErrorV1::StorageUnavailable)?;

    let target_reference_id = match transfer {
        Ok(reference_id) => reference_id,
        Err(error) => match blob_transfer_failure(error) {
            BlobCustodyTransferFailureV1::PolicyRejected => {
                persistence
                    .fail_body_custody_transfer(&claimed, now_unix_seconds)
                    .await
                    .map_err(storage_error)?;
                return Ok(true);
            }
            BlobCustodyTransferFailureV1::RetryPending => {
                return Err(CommunicationsCustodyWorkerErrorV1::RetryPending);
            }
        },
    };
    let blob_ref = format!(
        "blob-content:{}",
        target_reference_id
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    persistence
        .complete_body_custody_transfer(
            &claimed,
            CommunicationBodyBlobReferenceV1 {
                blob_ref,
                reference_id: target_reference_id,
                declared_bytes: claimed.declared_bytes,
                sha256: claimed.plaintext_sha256,
            },
            now_unix_seconds,
        )
        .await
        .map_err(storage_error)?;
    Ok(true)
}

fn storage_error(
    _: CommunicationsBodyCustodyTransferErrorV1,
) -> CommunicationsCustodyWorkerErrorV1 {
    CommunicationsCustodyWorkerErrorV1::StorageUnavailable
}

fn blob_transfer_failure(error: BlobClientError) -> BlobCustodyTransferFailureV1 {
    match error {
        BlobClientError::Rejected(_) => BlobCustodyTransferFailureV1::PolicyRejected,
        BlobClientError::InvalidSocketPath
        | BlobClientError::InvalidTimeout
        | BlobClientError::Connect(_)
        | BlobClientError::Io(_)
        | BlobClientError::FrameTooLarge
        | BlobClientError::InvalidFrame
        | BlobClientError::InvalidResponse
        | BlobClientError::InvalidSessionRequest
        | BlobClientError::Unavailable => BlobCustodyTransferFailureV1::RetryPending,
    }
}

#[cfg(test)]
mod tests {
    use super::{BlobCustodyTransferFailureV1, blob_transfer_failure};
    use hermes_blob_client::BlobClientError;

    #[test]
    fn blob_unavailability_keeps_custody_transfer_pending() {
        assert_eq!(
            blob_transfer_failure(BlobClientError::Unavailable),
            BlobCustodyTransferFailureV1::RetryPending,
        );
    }

    #[test]
    fn rejected_custody_transfer_remains_terminal() {
        assert_eq!(
            blob_transfer_failure(BlobClientError::Rejected("denied".to_owned())),
            BlobCustodyTransferFailureV1::PolicyRejected,
        );
    }
}
