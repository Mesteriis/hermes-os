//! Managed Zulip integration runtime.
//!
//! This crate composes provider-local HTTP, persistence and the public
//! Communications ingress contract. It never reaches Communications storage.

mod communications_outbox;
pub mod blob;
pub mod client_port;
pub mod managed;

use hermes_communications_ingress::{
    BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1,
    CommunicationObservationDraft, ObservationEnvelopeBuildErrorV1, ObservationEnvelopeContextV1,
    build_observation_outbox_record_v1, with_admitted_body_blob, with_body_admission_failure,
};
use hermes_runtime_protocol::v1::BlobDataOperationV1;
use hermes_zulip_api::{
    ZulipCommandOperationStatusV1, ZulipCommandReceiptV1, ZulipCommandV1, ZulipEventQueueV1,
    ZulipPolledEventV1, command_account_id, command_fingerprint_bytes, command_operation_id,
};
use hermes_zulip_core::{ZulipCoreError, observation_drafts};
use hermes_zulip_http::{
    ZulipHttpConfigV1, ZulipHttpErrorV1, execute_command as execute_http_command,
    download_user_upload, poll_event_queue, register_event_queue, upload_file,
};
use hermes_zulip_persistence::{
    ZulipCommandOperationStateV1, ZulipDurablePersistence, ZulipDurablePersistenceError,
    ZulipQueueCursorV1,
};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

pub use communications_outbox::{
    ZulipCommunicationsOutboxRelayError, relay_communications_outbox_once,
};

pub const PACKAGE: &str = "hermes-zulip-runtime";

pub mod settings;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipRuntimeIdentityV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

#[derive(Clone)]
pub struct ZulipRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
    pub api_key_revision: u64,
}

#[derive(Debug)]
pub enum ZulipRuntimeErrorV1 {
    Core(ZulipCoreError),
    Envelope(ObservationEnvelopeBuildErrorV1),
    Http(ZulipHttpErrorV1),
    Persistence(ZulipDurablePersistenceError),
    Credential,
    OperationAlreadyKnown,
}

/// Records a command before any worker may contact the provider.
pub async fn submit_command(
    durable: &ZulipDurablePersistence,
    command: &ZulipCommandV1,
    requested_at_unix_seconds: i64,
) -> Result<ZulipCommandReceiptV1, ZulipRuntimeErrorV1> {
    let command_sha256: [u8; 32] = Sha256::digest(command_fingerprint_bytes(command)).into();
    let operation_id = command_operation_id(command);
    if !durable
        .enqueue_command_operation(
            operation_id,
            command_account_id(command),
            &command_sha256,
            &hermes_zulip_api::client_wire::encode_command(command),
            requested_at_unix_seconds,
        )
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?
    {
        return Err(ZulipRuntimeErrorV1::OperationAlreadyKnown);
    }
    Ok(ZulipCommandReceiptV1 { operation_id: operation_id.to_owned(), account_id: command_account_id(command).to_owned() })
}

/// Claims and executes at most one previously persisted command. A command is
/// never implicitly retried after the durable dispatch fence has been written.
pub async fn execute_next_command(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    dispatched_at_unix_seconds: i64,
    completed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    execute_next_command_with_blob(
        durable,
        config,
        None,
        None,
        |_, _| Err(ZulipRuntimeErrorV1::Credential),
        dispatched_at_unix_seconds,
        completed_at_unix_seconds,
    )
    .await
}

pub async fn execute_next_command_with_blob(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    blob_materializer: Option<&Mutex<Option<blob::ZulipBlobMaterializer<hermes_blob_client::BlobDataClient>>>>,
    blob_write_materializer: Option<&Mutex<Option<blob::ZulipBlobWriteMaterializer<hermes_blob_client::BlobDataClient>>>>,
    mut authorize_blob: impl FnMut(&ZulipCommandV1, BlobDataOperationV1) -> Result<(), ZulipRuntimeErrorV1>,
    dispatched_at_unix_seconds: i64,
    completed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    let Some(queued) = durable.claim_next_command(dispatched_at_unix_seconds).await.map_err(ZulipRuntimeErrorV1::Persistence)? else {
        return Ok(false);
    };
    let command = hermes_zulip_api::client_wire::decode_command(&queued.exact_command_bytes)
        .map_err(|_| ZulipRuntimeErrorV1::Persistence(ZulipDurablePersistenceError::InvalidRow))?;
    let command_sha256: [u8; 32] = Sha256::digest(command_fingerprint_bytes(&command)).into();
    if queued.operation_id != command_operation_id(&command)
        || queued.account_id != command_account_id(&command)
        || queued.command_sha256 != command_sha256
    {
        return Err(ZulipRuntimeErrorV1::Persistence(ZulipDurablePersistenceError::InvalidRow));
    }
    let (execution, provider_upload_started, completed_blob_ref) = match &command {
        ZulipCommandV1::SendStreamWithUpload { stream, topic, content, blob, filename, .. } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationReadRangeV1)?;
            let bytes = take_blob_bytes(blob_materializer, &blob.blob_ref)?;
            let uri = upload_file(config, filename, &bytes).await.map_err(ZulipRuntimeErrorV1::Http)?;
            let send = ZulipCommandV1::SendStream {
                operation_id: command_operation_id(&command).to_owned(), account_id: command_account_id(&command).to_owned(),
                stream: stream.clone(), topic: topic.clone(), content: content_with_upload_uri(content, &uri),
            };
            (execute_http_command(config, &send).await, true, None)
        }
        ZulipCommandV1::SendDirectWithUpload { recipients, content, blob, filename, .. } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationReadRangeV1)?;
            let bytes = take_blob_bytes(blob_materializer, &blob.blob_ref)?;
            let uri = upload_file(config, filename, &bytes).await.map_err(ZulipRuntimeErrorV1::Http)?;
            let send = ZulipCommandV1::SendDirect {
                operation_id: command_operation_id(&command).to_owned(), account_id: command_account_id(&command).to_owned(),
                recipients: recipients.clone(), content: content_with_upload_uri(content, &uri),
            };
            (execute_http_command(config, &send).await, true, None)
        }
        ZulipCommandV1::DownloadAttachment { upload_path, blob, .. } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationWriteV1)?;
            let bytes = download_user_upload(config, upload_path).await.map(|(bytes, _)| bytes).map_err(ZulipRuntimeErrorV1::Http)?;
            write_downloaded_blob(blob_write_materializer, &blob.blob_ref, bytes)?;
            (Ok(hermes_zulip_http::ZulipHttpResponseV1 { status: 200, provider_message_id: None }), true, Some(blob.blob_ref.as_str()))
        }
        _ => (execute_http_command(config, &command).await, false, None),
    };
    match execution {
        Ok(response) => {
            durable
                .complete_command_operation(
                    &queued.operation_id,
                    &queued.command_sha256,
                    ZulipCommandOperationStateV1::Accepted,
                    response.provider_message_id,
                    completed_blob_ref,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(ZulipRuntimeErrorV1::Persistence)?;
            Ok(true)
        }
        Err(error @ (ZulipHttpErrorV1::InvalidCommand | ZulipHttpErrorV1::Rejected)) if !provider_upload_started => {
            durable
                .complete_command_operation(
                    &queued.operation_id,
                    &queued.command_sha256,
                    ZulipCommandOperationStateV1::Rejected,
                    None,
                    None,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(ZulipRuntimeErrorV1::Persistence)?;
            Err(ZulipRuntimeErrorV1::Http(error))
        }
        Err(error) => Err(ZulipRuntimeErrorV1::Http(error)),
    }
}

fn write_downloaded_blob(
    materializer: Option<&Mutex<Option<blob::ZulipBlobWriteMaterializer<hermes_blob_client::BlobDataClient>>>>,
    blob_ref: &str,
    bytes: Vec<u8>,
) -> Result<(), ZulipRuntimeErrorV1> {
    let materializer = materializer.ok_or(ZulipRuntimeErrorV1::Credential)?;
    materializer.lock().map_err(|_| ZulipRuntimeErrorV1::Credential)?.as_mut().ok_or(ZulipRuntimeErrorV1::Credential)?.write_download(blob_ref, bytes)
}

fn take_blob_bytes(
    materializer: Option<&Mutex<Option<blob::ZulipBlobMaterializer<hermes_blob_client::BlobDataClient>>>>,
    blob_ref: &str,
) -> Result<Vec<u8>, ZulipRuntimeErrorV1> {
    let materializer = materializer.ok_or(ZulipRuntimeErrorV1::Credential)?;
    materializer.lock().map_err(|_| ZulipRuntimeErrorV1::Credential)?
        .as_mut().ok_or(ZulipRuntimeErrorV1::Credential)?
        .take_bytes(blob_ref)
}

fn content_with_upload_uri(content: &str, upload_uri: &str) -> String {
    format!("{content}\n{upload_uri}")
}

pub async fn command_operation_status(
    durable: &ZulipDurablePersistence,
    operation_id: &str,
) -> Result<Option<ZulipCommandOperationStatusV1>, ZulipRuntimeErrorV1> {
    durable
        .command_operation_status(operation_id)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

impl ZulipRuntimeIdentityV1 {
    #[must_use]
    pub fn observation_context(
        &self,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> ObservationEnvelopeContextV1 {
        ObservationEnvelopeContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            module_id: "zulip-runtime".to_owned(),
            recorded_at_unix_seconds,
            recorded_at_nanos,
        }
    }
}

pub fn api_key_revision(admission: &ZulipRuntimeAdmissionV1) -> Result<u64, ZulipRuntimeErrorV1> {
    (admission.api_key_revision != 0)
        .then_some(admission.api_key_revision)
        .ok_or(ZulipRuntimeErrorV1::Core(ZulipCoreError::CredentialLeaseRejected))
}

pub async fn acquire_event_queue(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
) -> Result<ZulipEventQueueV1, ZulipRuntimeErrorV1> {
    match durable
        .current_cursor(&config.account.account_id)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?
    {
        Some(cursor) => Ok(ZulipEventQueueV1 {
            queue_id: cursor.queue_id,
            last_event_id: cursor.last_event_id,
        }),
        None => register_event_queue(config)
            .await
            .map_err(ZulipRuntimeErrorV1::Http),
    }
}

pub async fn poll_once<F>(
    durable: &ZulipDurablePersistence,
    identity: &ZulipRuntimeIdentityV1,
    config: &ZulipHttpConfigV1,
    queue: &mut ZulipEventQueueV1,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
    body_admitter: &mut F,
) -> Result<usize, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let events = poll_event_queue(config, queue)
        .await
        .map_err(ZulipRuntimeErrorV1::Http)?;
    let mut accepted = 0;
    for event in events {
        if accept_polled_event(
            durable,
            identity,
            &config.account.account_id,
            &queue.queue_id,
            &event,
            recorded_at_unix_seconds,
            recorded_at_nanos,
            body_admitter,
        )
        .await?
        {
            accepted += 1;
        }
        queue.last_event_id = queue.last_event_id.max(event.event_id);
    }
    Ok(accepted)
}

pub async fn accept_polled_event<F>(
    durable: &ZulipDurablePersistence,
    identity: &ZulipRuntimeIdentityV1,
    account_id: &str,
    queue_id: &str,
    event: &ZulipPolledEventV1,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
    body_admitter: &mut F,
) -> Result<bool, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let cursor = ZulipQueueCursorV1 {
        account_id: account_id.to_owned(),
        queue_id: queue_id.to_owned(),
        last_event_id: event.event_id,
    };
    if event.observations.is_empty() {
        return durable
            .advance_cursor(&cursor)
            .await
            .map_err(ZulipRuntimeErrorV1::Persistence);
    }
    let mut records = Vec::new();
    for observation in &event.observations {
        for draft in observation_drafts(observation).map_err(ZulipRuntimeErrorV1::Core)? {
            let draft = admit_message_body(draft, observation, body_admitter)?;
            records.push(build_observation_outbox_record_v1(
                &draft, &identity.observation_context(recorded_at_unix_seconds, recorded_at_nanos),
            ).map_err(ZulipRuntimeErrorV1::Envelope)?);
        }
    }
    durable
        .advance_cursor_and_enqueue_many(&cursor, &records, recorded_at_unix_seconds)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

fn admit_message_body<F>(
    draft: CommunicationObservationDraft,
    event: &hermes_zulip_api::ZulipEventV1,
    body_admitter: &mut F,
) -> Result<CommunicationObservationDraft, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let hermes_zulip_api::ZulipEventV1::Message { content: Some(content), .. } = event else {
        return Ok(draft);
    };
    if draft.body != BodyAvailabilityV1::Unavailable { return Ok(draft); }
    if content.trim().is_empty() || content.len() > 256 * 1024 {
        return with_body_admission_failure(draft, BodyAdmissionFailureV1::SizeLimitExceeded)
            .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent));
    }
    match body_admitter(content.as_bytes()) {
        Ok(receipt) => {
            let mut admitted = draft;
            admitted.body = BodyAvailabilityV1::AdmittedBlob;
            with_admitted_body_blob(admitted, receipt)
                .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent))
        }
        Err(failure) => with_body_admission_failure(draft, failure)
            .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent)),
    }
}
