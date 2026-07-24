//! Executes one signed direct Blob request without exposing keys to callers.

use std::future::Future;
use std::os::unix::net::UnixStream;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_blob_runtime::{
    lease::BlobKeyLeaseV1,
    storage::{BlobContentLifecycleStore, BlobCustodyTransferRequestV1},
    vault::{BlobContentKeyFenceV1, BlobVaultKeyLeaseAdapterV1, BlobVaultRoutePortV1},
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1, BlobDataRequestV1, BlobDataResponseV1, blob_data_request_v1::Operation,
};
use prost::Message;

use super::{
    framing,
    session::{
        BlobDataSessionVerifierV1, VerifiedBlobCustodyTransferV1, VerifiedBlobDataSessionV1,
    },
};

pub(crate) struct BlobDataService<R> {
    store: BlobContentLifecycleStore,
    verifier: BlobDataSessionVerifierV1,
    keys: BlobVaultKeyLeaseAdapterV1<R>,
}

impl<R> BlobDataService<R>
where
    R: BlobVaultRoutePortV1 + Send,
{
    pub(crate) fn new(
        store: BlobContentLifecycleStore,
        verifier: BlobDataSessionVerifierV1,
        keys: BlobVaultKeyLeaseAdapterV1<R>,
    ) -> Self {
        Self {
            store,
            verifier,
            keys,
        }
    }

    pub(crate) fn serve_one(&mut self, stream: &mut UnixStream) -> Result<(), ()> {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        let response = framing::read_frame(stream)
            .and_then(|bytes| BlobDataRequestV1::decode(bytes.as_slice()).map_err(|_| ()))
            .and_then(|request| self.handle(request))
            .unwrap_or_else(|_| BlobDataResponseV1 {
                plaintext: Vec::new(),
                accepted: false,
                error_code: "data_request_denied".to_owned(),
            });
        framing::write_frame(stream, &response.encode_to_vec())
    }

    fn handle(&mut self, request: BlobDataRequestV1) -> Result<BlobDataResponseV1, ()> {
        let now = now_unix_ms()?;
        if let Some(Operation::CustodyTransfer(transfer)) = request.operation.as_ref() {
            let transfer = self.verifier.verify_custody_transfer(
                transfer.grant.clone().ok_or(())?,
                &transfer.channel_binding,
                now,
            )?;
            return self.custody_transfer(transfer, now);
        }
        let (operation, expected) = match request.operation.as_ref() {
            Some(Operation::Write(_)) => (
                request.operation,
                BlobDataOperationV1::BlobDataOperationWriteV1,
            ),
            Some(Operation::ReadRange(_)) => (
                request.operation,
                BlobDataOperationV1::BlobDataOperationReadRangeV1,
            ),
            Some(Operation::CustodyTransfer(_)) => return Err(()),
            None => return Err(()),
        };
        let grant = request.grant.ok_or(())?;
        let session = self
            .verifier
            .verify(grant, &request.channel_binding, expected, now)
            .map_err(|_| developer_denied("session"))?;
        let lease = self
            .content_key(&session, now)
            .map_err(|_| developer_denied("content_key"))?;
        match operation.ok_or(())? {
            Operation::Write(write) => {
                self.store
                    .write_new(
                        session.reference(),
                        session.access(),
                        session.quota(),
                        &lease,
                        &write.plaintext,
                        now,
                    )
                    .map_err(|_| developer_denied("write"))?;
                Ok(BlobDataResponseV1 {
                    plaintext: Vec::new(),
                    accepted: true,
                    error_code: String::new(),
                })
            }
            Operation::ReadRange(read) => {
                let range = hermes_blob_protocol::BlobRangeV1::new(
                    read.start,
                    read.end_exclusive,
                    session.reference().declared_size(),
                )
                .map_err(|_| ())?;
                let plaintext = self
                    .store
                    .read_range(session.reference(), session.access(), &lease, range, now)
                    .map_err(|_| developer_denied("read"))?;
                Ok(BlobDataResponseV1 {
                    plaintext,
                    accepted: true,
                    error_code: String::new(),
                })
            }
            Operation::CustodyTransfer(_) => Err(()),
        }
    }

    fn content_key(
        &mut self,
        session: &VerifiedBlobDataSessionV1,
        now: u64,
    ) -> Result<BlobKeyLeaseV1, ()> {
        let fence = BlobContentKeyFenceV1::new(session.access().clone(), session.key_revision())
            .map_err(|_| ())?;
        complete_immediately(
            self.keys
                .ensure_content_key(session.reference(), &fence, now),
        )
        .map_err(|_| ())?
        .map_err(|_| ())
    }

    fn custody_transfer(
        &mut self,
        transfer: VerifiedBlobCustodyTransferV1,
        now: u64,
    ) -> Result<BlobDataResponseV1, ()> {
        let source_key = self.content_key_for(
            transfer.source_reference(),
            transfer.source_access(),
            transfer.source_key_revision(),
            now,
        )?;
        let target_key = self.content_key_for(
            transfer.target_reference(),
            transfer.target_access(),
            transfer.target_key_revision(),
            now,
        )?;
        self.store
            .custody_transfer(BlobCustodyTransferRequestV1 {
                source_reference: transfer.source_reference(),
                source_access: transfer.source_access(),
                source_lease: &source_key,
                target_reference: transfer.target_reference(),
                target_access: transfer.target_access(),
                target_quota: transfer.target_quota(),
                target_lease: &target_key,
                expected_plaintext_sha256: transfer.expected_plaintext_sha256(),
                now_unix_ms: now,
            })
            .map_err(|_| ())?;
        Ok(BlobDataResponseV1 {
            plaintext: Vec::new(),
            accepted: true,
            error_code: String::new(),
        })
    }

    fn content_key_for(
        &mut self,
        reference: &hermes_blob_protocol::BlobRefV1,
        access: &hermes_blob_protocol::BlobAccessFenceV1,
        key_revision: u64,
        now: u64,
    ) -> Result<BlobKeyLeaseV1, ()> {
        let fence = BlobContentKeyFenceV1::new(access.clone(), key_revision).map_err(|_| ())?;
        complete_immediately(self.keys.ensure_content_key(reference, &fence, now))
            .map_err(|_| ())?
            .map_err(|_| ())
    }
}

fn developer_denied(stage: &str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_blob_data_request_denied stage={stage}");
    }
}

fn now_unix_ms() -> Result<u64, ()> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_millis()
        .try_into()
        .map_err(|_| ())
}

fn complete_immediately<T>(future: impl Future<Output = T>) -> Result<T, ()> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = std::pin::pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => Ok(value),
        Poll::Pending => Err(()),
    }
}
