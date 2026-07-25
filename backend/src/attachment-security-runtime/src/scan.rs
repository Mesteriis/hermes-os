//! Receipt-bound Blob materialization and loopback ClamAV scan adapter.

use std::{os::unix::net::UnixStream, time::Duration};

use hermes_attachment_security_clamav::{
    ClamAvInstreamLimitsV1, ClamAvLoopbackEndpointV1, ClamAvTimeoutsV1, scan_clamav_loopback_v1,
};
use hermes_attachment_security_core::ScannerOutcomeV1;
use hermes_attachment_security_persistence::ClaimedAttachmentSecurityScanJobV1;
use hermes_blob_client::{
    BlobDataClient, ManagedBlobSessionRequestV1, request_managed_blob_session_v2,
};
use hermes_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};

use crate::{
    admission::ATTACHMENT_SECURITY_BLOB_READ_CAPABILITY_ID,
    settings::AttachmentSecurityRuntimeSettingsV1,
};

const CLAMAV_CHUNK_BYTES: u32 = 64 * 1024;
const CLAMAV_MAX_RESPONSE_BYTES: u32 = 4 * 1024;

pub struct AttachmentSecurityScannerV1 {
    endpoint: ClamAvLoopbackEndpointV1,
    limits: ClamAvInstreamLimitsV1,
    timeouts: ClamAvTimeoutsV1,
}

impl AttachmentSecurityScannerV1 {
    pub fn new(
        settings: AttachmentSecurityRuntimeSettingsV1,
    ) -> Result<Self, AttachmentSecurityScanAdapterErrorV1> {
        Ok(Self {
            endpoint: ClamAvLoopbackEndpointV1::new(settings.clamav_port)
                .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
            limits: ClamAvInstreamLimitsV1::new(
                settings.max_scan_bytes,
                CLAMAV_CHUNK_BYTES,
                CLAMAV_MAX_RESPONSE_BYTES,
            )
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
            timeouts: ClamAvTimeoutsV1::new(
                Duration::from_millis(settings.clamav_connect_timeout_millis),
                Duration::from_millis(settings.clamav_io_timeout_millis),
            )
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
        })
    }

    pub fn scan_claimed(
        &self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
    ) -> Result<ScannerOutcomeV1, AttachmentSecurityScanAdapterErrorV1> {
        let bytes = read_blob(control_channel, claimed)?;
        scan_clamav_loopback_v1(
            self.endpoint,
            &bytes,
            claimed.job.declared_size,
            claimed.job.blob_receipt_sha256,
            self.limits,
            self.timeouts,
        )
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Unavailable)
    }
}

fn read_blob(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentSecurityScanJobV1,
) -> Result<Vec<u8>, AttachmentSecurityScanAdapterErrorV1> {
    if prepare_blocking_control_channel(control_channel).is_err() {
        let _ = restore_nonblocking_control_channel(control_channel);
        return Err(AttachmentSecurityScanAdapterErrorV1::Unavailable);
    }
    let result = (|| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: ATTACHMENT_SECURITY_BLOB_READ_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &claimed.job.blob_reference_id,
                declared_size: claimed.job.declared_size,
                backup_class: 1,
                receipt_sha256: Some(&claimed.job.blob_receipt_sha256),
            },
        )
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Unavailable)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    claimed.job.declared_size,
                )
            })
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Unavailable)
    })();
    let restored = restore_nonblocking_control_channel(control_channel);
    match (result, restored) {
        (Ok(bytes), Ok(())) => Ok(bytes),
        _ => Err(AttachmentSecurityScanAdapterErrorV1::Unavailable),
    }
}

fn prepare_blocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityScanAdapterErrorV1> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            channel
                .inner_mut()
                .set_read_timeout(Some(Duration::from_secs(5)))
        })
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Unavailable)
}

fn restore_nonblocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityScanAdapterErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Unavailable)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityScanAdapterErrorV1 {
    InvalidConfiguration,
    Unavailable,
}
