//! Kernel-admitted Mail runtime bootstrap. No CLI, provider, or domain fallback exists here.

use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_blob_client::{BlobDataClient, request_managed_blob_session};
use hermes_communications_ingress::{ObservationEnvelopeContextV1, build_observation_outbox_record_v1};
use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV1, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1,
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1,
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeClientDeliveryResponseV1,
    ManagedStorageRuntimeConfigurationV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use hermes_vault_protocol::{DEFAULT_LEASE_TTL_SECONDS, SecretClassV1};
use hermes_runtime_protocol::validation::module_client::{validate_module_client_request_v1, validate_module_client_response_v1};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::MailRuntimeAdmission;
use crate::communications_outbox::{MailCommunicationsOutboxRelayError, relay_communications_outbox_once};
use hermes_mail_api::{
    MailCredentialPurpose, MailInboundTransportV1, MailSendMailRequestV1, OutgoingMailV1,
    valid_account_configuration, valid_port,
};
use hermes_communications_ingress::{
    AttachmentDispositionV1, BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1,
    CommunicationObservationDraft, ProviderProvenanceV1, with_admitted_body_blob,
    with_body_admission_failure,
};
use hermes_mail_core::{
    bounded_window, compose_rfc822, draft_attachment_ingress_observation, draft_delivery_observation,
    draft_ingress_observation_with_body,
    validate_sync_request,
};
use hermes_mail_core::rfc822::{AttachmentDispositionV1 as Rfc822AttachmentDispositionV1, attachment_metadata, direct_plain_text_body};
use hermes_mail_persistence::MailDurablePersistence;
use hermes_mail_gmail::{GmailAdapterErrorV1, GmailApiClientV1, GmailListMessagesRequestV1, decode_raw_rfc822, history_message_ids};

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct MailAdmittedRuntime {
    pub control_channel: UnixStream,
    pub durable: MailDurablePersistence,
    inbound_credential: MailInboundCredentialV1,
    smtp_password: Option<Zeroizing<Vec<u8>>>,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    account: hermes_mail_api::MailAccountConfigurationV1,
    runtime_instance_id: String,
    runtime_generation: u64,
}

enum MailInboundCredentialV1 {
    ImapPassword(Zeroizing<Vec<u8>>),
    GmailAccessToken(Zeroizing<Vec<u8>>),
}

enum GmailHistorySyncError {
    Expired,
    Runtime(MailBootstrapError),
}

#[derive(Debug)]
pub enum MailBootstrapError {
    Admission,
    Control,
    Storage,
    Credential,
    Persistence,
    Provider,
    EventHub,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    mut control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &MailRuntimeAdmission,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<MailAdmittedRuntime, MailBootstrapError> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.runtime_instance_id.trim().is_empty()
        || !valid_account_configuration(&admission.account)
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
    {
        return Err(MailBootstrapError::Admission);
    }
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| MailBootstrapError::Control)?;
    write_frame(
        &mut control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                descriptor_bytes,
                settings_schema_bytes,
            })),
        }
        .encode_to_vec(),
    )?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut control_channel)?.as_slice())
        .map_err(|_| MailBootstrapError::Control)?;
    let (registration_id, runtime_generation, grant_epoch) = match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0
                && value.grant_epoch != 0 =>
        {
            (value.registration_id, value.runtime_generation, value.grant_epoch)
        }
        _ => return Err(MailBootstrapError::Admission),
    };
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(MailBootstrapError::Admission);
    }
    write_frame(
        &mut control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                registration_id,
                runtime_generation,
                grant_epoch,
            })),
        }
        .encode_to_vec(),
    )?;
    control_channel
        .set_read_timeout(None)
        .and_then(|_| control_channel.set_write_timeout(None))
        .map_err(|_| MailBootstrapError::Control)?;

    let provider_context = provider_credential_context(admission, &storage_configuration)?;
    let mut provider_credentials = ManagedProviderCredentialClientV1::new(
        control_channel.try_clone().map_err(|_| MailBootstrapError::Control)?,
    );
    let inbound_credential = match &admission.account.inbound {
        MailInboundTransportV1::Imap(_) => {
            let revision = credential_revision(admission, MailCredentialPurpose::ImapPassword)?
                .ok_or(MailBootstrapError::Admission)?;
            MailInboundCredentialV1::ImapPassword(provider_credentials
                .resolve(
                    &provider_context,
                    &admission.configuration_instance_id,
                    MailCredentialPurpose::ImapPassword.as_str(),
                    revision,
                    DEFAULT_LEASE_TTL_SECONDS,
                    SecretClassV1::ProviderCredential,
                )
                .map_err(map_provider_credential_error)?)
        }
        MailInboundTransportV1::Gmail(_) => {
            let revision = credential_revision(admission, MailCredentialPurpose::GmailAccessToken)?
                .ok_or(MailBootstrapError::Admission)?;
            MailInboundCredentialV1::GmailAccessToken(provider_credentials
                .resolve(
                    &provider_context,
                    &admission.configuration_instance_id,
                    MailCredentialPurpose::GmailAccessToken.as_str(),
                    revision,
                    DEFAULT_LEASE_TTL_SECONDS,
                    SecretClassV1::ProviderCredential,
                )
                .map_err(map_provider_credential_error)?)
        }
    };
    let smtp_password = match credential_revision(admission, MailCredentialPurpose::SmtpPassword)? {
        Some(revision) => Some(provider_credentials
            .resolve(
                &provider_context,
                &admission.configuration_instance_id,
                MailCredentialPurpose::SmtpPassword.as_str(),
                revision,
                DEFAULT_LEASE_TTL_SECONDS,
                SecretClassV1::ProviderCredential,
            )
            .map_err(map_provider_credential_error)?),
        None => None,
    };

    let binding = storage_binding(&storage_configuration, admission)?;
    let storage_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| MailBootstrapError::Storage)?,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let mut storage_leases = StorageVaultLeaseAdapterV1::new(
        InheritedKernelVaultRouteV1::new(
            control_channel.try_clone().map_err(|_| MailBootstrapError::Control)?,
        ),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|_| MailBootstrapError::Credential)?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|_| MailBootstrapError::Credential)?;
    let password = std::str::from_utf8(&password).map_err(|_| MailBootstrapError::Credential)?;
    let durable = MailDurablePersistence::connect_runtime(
        &binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        password,
    )
    .await
    .map_err(|_| MailBootstrapError::Persistence)?;
    durable.initialize().await.map_err(|_| MailBootstrapError::Persistence)?;
    let event_access = request_managed_runtime_event_access(
        &mut control_channel,
        &admission.logical_owner_id,
        &admission.module_registration_id,
        &admission.runtime_instance_id,
        admission.runtime_generation,
        admission.grant_epoch,
        event_credential_revision,
    )
    .map_err(|_| MailBootstrapError::EventHub)?;
    let identity = RuntimeNatsIdentity::new(
        admission.runtime_instance_id.clone(),
        admission.runtime_generation,
        admission.grant_epoch,
    )
    .map_err(|_| MailBootstrapError::EventHub)?;
    let event_publish_permit = event_access
        .publish_permit(
            &admission.module_registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| MailBootstrapError::EventHub)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| MailBootstrapError::EventHub)?;
    control_channel.set_nonblocking(true).map_err(|_| MailBootstrapError::Control)?;
    Ok(MailAdmittedRuntime {
        control_channel,
        durable,
        inbound_credential,
        smtp_password,
        event_connection,
        event_publish_permit,
        account: admission.account.clone(),
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
    })
}

impl MailAdmittedRuntime {
    pub async fn try_handle_client_delivery(&mut self) -> Result<bool, MailBootstrapError> {
        let Some(frame) = peek_complete_frame(&self.control_channel)? else { return Ok(false); };
        let request = ManagedRuntimeClientDeliveryRequestV1::decode(frame.as_slice())
            .map_err(|_| MailBootstrapError::Control)?
            .request
            .ok_or(MailBootstrapError::Control)?;
        validate_module_client_request_v1(&request).map_err(|_| MailBootstrapError::Control)?;
        if read_frame(&mut self.control_channel)? != frame { return Err(MailBootstrapError::Control); }
        let payload = crate::client_port::handle_client_request(self, &request.encode_to_vec())
            .await
            .map_err(|_| MailBootstrapError::Provider)?;
        let response = hermes_runtime_protocol::v1::ModuleClientResponseV1::decode(payload.as_slice())
            .map_err(|_| MailBootstrapError::Provider)?;
        validate_module_client_response_v1(&response).map_err(|_| MailBootstrapError::Provider)?;
        write_frame(&mut self.control_channel, &ManagedRuntimeClientDeliveryResponseV1 { response: Some(response) }.encode_to_vec())?;
        Ok(true)
    }

    pub async fn send_configured_mail(
        &mut self,
        request: &MailSendMailRequestV1,
    ) -> Result<u16, MailBootstrapError> {
        let message = OutgoingMailV1 {
            operation_id: request.operation_id.clone(),
            connection_id: self.account.connection_id.clone(),
            provider_conversation_id: request.provider_conversation_id.clone(),
            recipients: request.recipients.clone(),
            subject: request.subject.clone(),
            text_body: request.text_body.clone(),
        };
        let account = self.account.clone();
        match account.inbound {
            MailInboundTransportV1::Imap(_) => self.send_mail_via_smtp(
                account.smtp_endpoint.as_ref().ok_or(MailBootstrapError::Admission)?,
                &message,
            ).await,
            MailInboundTransportV1::Gmail(configuration) => self.send_mail_via_gmail(
                &configuration.user_id,
                &configuration.from_address,
                &message,
            ).await,
        }
    }

    async fn send_mail_via_smtp(
        &mut self,
        endpoint: &hermes_mail_api::SmtpEndpointV1,
        message: &OutgoingMailV1,
    ) -> Result<u16, MailBootstrapError> {
        let password = self.smtp_password.as_deref().ok_or(MailBootstrapError::Credential)?;
        let password = std::str::from_utf8(password).map_err(|_| MailBootstrapError::Credential)?;
        self.send_mail(message, &endpoint.from_address, ProviderProvenanceV1::MailSmtp, |rfc822_message| async move {
            hermes_mail_smtp::send_implicit_tls(endpoint, message, password, &rfc822_message)
                .await
                .map(|receipt| receipt.response_code)
        }).await
    }

    async fn send_mail_via_gmail(
        &self,
        user_id: &str,
        from_address: &str,
        message: &OutgoingMailV1,
    ) -> Result<u16, MailBootstrapError> {
        let MailInboundCredentialV1::GmailAccessToken(access_token) = &self.inbound_credential else {
            return Err(MailBootstrapError::Credential);
        };
        let access_token = std::str::from_utf8(access_token).map_err(|_| MailBootstrapError::Credential)?;
        self.send_mail(message, from_address, ProviderProvenanceV1::MailGmail, |rfc822_message| async move {
            let client = hermes_mail_gmail::GmailApiClientV1::new(user_id)
                .map_err(|_| hermes_mail_gmail::GmailAdapterErrorV1::Transport)?;
            client.send_raw_message(access_token, rfc822_message.as_bytes(), Some(&message.provider_conversation_id))
                .await
                .map(|_| 200)
        }).await
    }

    async fn send_mail<F, Fut, E>(
        &self,
        message: &OutgoingMailV1,
        from_address: &str,
        provider: ProviderProvenanceV1,
        execute: F,
    ) -> Result<u16, MailBootstrapError>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<u16, E>>,
    {
        let rfc822_message = compose_rfc822(from_address, message).map_err(|_| MailBootstrapError::Admission)?;
        let rfc822_sha256: [u8; 32] = Sha256::digest(rfc822_message.as_bytes()).into();
        let attempted_at = current_unix_seconds()?;
        let started = self.durable
            .begin_delivery_attempt(
                &message.operation_id,
                &message.connection_id,
                &rfc822_sha256,
                attempted_at,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        if !started {
            return Err(MailBootstrapError::Admission);
        }
        let response_code = match execute(rfc822_message).await {
            Ok(response_code) => response_code,
            Err(_) => {
                self.durable
                    .complete_delivery_rejected(
                        &message.operation_id,
                        &rfc822_sha256,
                        current_unix_seconds()?,
                    )
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                return Err(MailBootstrapError::Provider);
            }
        };
        let completed_at = current_unix_seconds()?;
        let observation = draft_delivery_observation(provider, message).map_err(|_| MailBootstrapError::Admission)?;
        let record = build_observation_outbox_record_v1(
            &observation,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: self.runtime_instance_id.clone(),
                runtime_generation: self.runtime_generation,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: completed_at,
                recorded_at_nanos: 0,
            },
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .complete_delivery_accepted(
                &message.operation_id,
                &rfc822_sha256,
                response_code,
                &record,
                completed_at,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        Ok(response_code)
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, MailCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }

    pub async fn sync_configured_inbox(
        &mut self,
        operation_id: &str,
    ) -> Result<usize, MailBootstrapError> {
        let account = self.account.clone();
        match account.inbound {
            MailInboundTransportV1::Imap(configuration) => self.sync_inbox(
                &account.connection_id,
                operation_id,
                &configuration.host,
                configuration.port,
                &configuration.username,
                account.sync_window,
                account.sync_windows,
            ).await,
            MailInboundTransportV1::Gmail(configuration) => self.sync_gmail_inbox(
                &account.connection_id,
                operation_id,
                &configuration.user_id,
                account.sync_window,
                account.sync_windows,
            ).await,
        }
    }

    async fn sync_inbox(
        &mut self,
        connection_id: &str,
        operation_id: &str,
        host: &str,
        port: u16,
        username: &str,
        window: u32,
        windows: u32,
    ) -> Result<usize, MailBootstrapError> {
        if connection_id.trim().is_empty()
            || operation_id.trim().is_empty()
            || username.trim().is_empty()
            || !valid_port(port)
        {
            return Err(MailBootstrapError::Admission);
        }
        validate_sync_request(host, port, 0).map_err(|_| MailBootstrapError::Admission)?;
        let plan = bounded_window(window, windows).map_err(|_| MailBootstrapError::Admission)?;
        let MailInboundCredentialV1::ImapPassword(password) = &self.inbound_credential else {
            return Err(MailBootstrapError::Credential);
        };
        let password = Zeroizing::new(password.to_vec());
        let password = std::str::from_utf8(&password)
            .map_err(|_| MailBootstrapError::Credential)?;
        let messages = hermes_mail_imap::sync_inbox(
            host,
            port,
            username,
            Some(password),
            plan.window,
            plan.windows,
        )
        .map_err(|_| MailBootstrapError::Provider)?
        .messages;
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_unix_seconds = i64::try_from(observed_at.as_secs())
            .map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos = i32::try_from(observed_at.subsec_nanos())
            .map_err(|_| MailBootstrapError::Provider)?;
        for message in &messages {
            let observation = self.draft_inbound_body_observation(
                &inbound_observation_id(
                    ProviderProvenanceV1::MailImap,
                    connection_id,
                    &message.uid.to_string(),
                    None,
                ),
                ProviderProvenanceV1::MailImap,
                connection_id,
                format!("{connection_id}:{}", message.uid),
                message.plain_text_body.clone(),
            )
            ?;
            let record = build_observation_outbox_record_v1(
                &observation,
                &ObservationEnvelopeContextV1 {
                    runtime_instance_id: self.runtime_instance_id.clone(),
                    runtime_generation: self.runtime_generation,
                    module_id: "mail-runtime".to_owned(),
                    recorded_at_unix_seconds: observed_at_unix_seconds,
                    recorded_at_nanos: observed_at_nanos,
                },
            )
            .map_err(|_| MailBootstrapError::Admission)?;
            self.durable
                .enqueue_communications_outbox(&record, observed_at_unix_seconds)
                .await
                .map_err(|_| MailBootstrapError::Persistence)?;
            for attachment in &message.attachments {
                let source_id = format!("{connection_id}:{}", message.uid);
                let media_id = format!("{}:{}", message.uid, attachment.part_id);
                let disposition = match attachment.disposition {
                    hermes_mail_imap::ImapAttachmentDisposition::Attachment => AttachmentDispositionV1::Attachment,
                    hermes_mail_imap::ImapAttachmentDisposition::Inline => AttachmentDispositionV1::Inline,
                };
                let observation = draft_attachment_ingress_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailImap,
                        connection_id,
                        &message.uid.to_string(),
                        Some(attachment.part_id),
                    ),
                    ProviderProvenanceV1::MailImap,
                    connection_id,
                    source_id,
                    media_id,
                    attachment.filename.clone(),
                    attachment.media_type.clone(),
                    attachment.declared_bytes,
                    disposition,
                )
                .map_err(|_| MailBootstrapError::Provider)?;
                let record = build_observation_outbox_record_v1(
                    &observation,
                    &ObservationEnvelopeContextV1 {
                        runtime_instance_id: self.runtime_instance_id.clone(),
                        runtime_generation: self.runtime_generation,
                        module_id: "mail-runtime".to_owned(),
                        recorded_at_unix_seconds: observed_at_unix_seconds,
                        recorded_at_nanos: observed_at_nanos,
                    },
                )
                .map_err(|_| MailBootstrapError::Admission)?;
                self.durable
                    .enqueue_communications_outbox(&record, observed_at_unix_seconds)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
            }
        }
        Ok(messages.len())
    }

    async fn sync_gmail_inbox(
        &mut self,
        connection_id: &str,
        operation_id: &str,
        user_id: &str,
        window: u32,
        windows: u32,
    ) -> Result<usize, MailBootstrapError> {
        if connection_id.trim().is_empty() || operation_id.trim().is_empty() {
            return Err(MailBootstrapError::Admission);
        }
        let plan = bounded_window(window, windows).map_err(|_| MailBootstrapError::Admission)?;
        let MailInboundCredentialV1::GmailAccessToken(token) = &self.inbound_credential else {
            return Err(MailBootstrapError::Credential);
        };
        let token = Zeroizing::new(token.to_vec());
        let token = std::str::from_utf8(&token).map_err(|_| MailBootstrapError::Credential)?;
        let max_results = u16::try_from(plan.window.min(500)).map_err(|_| MailBootstrapError::Admission)?;
        let client = GmailApiClientV1::new(user_id).map_err(|_| MailBootstrapError::Admission)?;
        let observed_at = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_unix_seconds = i64::try_from(observed_at.as_secs()).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos = i32::try_from(observed_at.subsec_nanos()).map_err(|_| MailBootstrapError::Provider)?;
        if let Some((start_history_id, page_token)) = self.durable.gmail_history_checkpoint(connection_id).await.map_err(|_| MailBootstrapError::Persistence)? {
            match self.sync_gmail_history_pages(connection_id, token, &client, &start_history_id, page_token, plan.windows, observed_at_unix_seconds, observed_at_nanos).await {
                Ok(observed_messages) => return Ok(observed_messages),
                Err(GmailHistorySyncError::Expired) => self.durable.clear_gmail_history_checkpoint(connection_id).await.map_err(|_| MailBootstrapError::Persistence)?,
                Err(GmailHistorySyncError::Runtime(error)) => return Err(error),
            }
        }
        let mut observed_messages = 0_usize;
        let (mut page_token, mut observed_history_id) = self.durable.gmail_sync_progress(connection_id).await.map_err(|_| MailBootstrapError::Persistence)?.map(|(page_token, observed_history_id)| (Some(page_token), observed_history_id)).unwrap_or((None, None));
        for _ in 0..plan.windows {
            let page = client
                .list_messages(token, &GmailListMessagesRequestV1 {
                    max_results,
                    page_token: page_token.clone(),
                    query: None,
                    label_ids: Vec::new(),
                })
                .await
                .map_err(|_| MailBootstrapError::Provider)?;
            let next_page_token = page.next_page_token.clone();
            let listed_messages = page.messages;
            let page_message_count = listed_messages.len();
            let (records, page_history_id) = self.gmail_message_records(connection_id, token, &client, listed_messages.into_iter().map(|message| message.id), observed_at_unix_seconds, observed_at_nanos).await?;
            observed_messages = observed_messages.saturating_add(page_message_count);
            observed_history_id = newer_gmail_history_id(observed_history_id.as_deref(), page_history_id.as_deref()).map(str::to_owned);
            self.durable
                .enqueue_communications_outbox_and_store_gmail_sync_progress(
                    &records,
                    connection_id,
                    next_page_token.as_deref(),
                    observed_history_id.as_deref(),
                    observed_at_unix_seconds,
                )
                .await
                .map_err(|_| MailBootstrapError::Persistence)?;
            let has_next_page = next_page_token.is_some();
            page_token = next_page_token;
            if !has_next_page {
                break;
            }
        }
        Ok(observed_messages)
    }

    async fn sync_gmail_history_pages(
        &mut self,
        connection_id: &str,
        token: &str,
        client: &GmailApiClientV1,
        start_history_id: &str,
        mut page_token: Option<String>,
        windows: u32,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<usize, GmailHistorySyncError> {
        let mut observed_messages = 0_usize;
        for _ in 0..windows {
            let page = match client.list_history(token, start_history_id, page_token.as_deref()).await {
                Ok(page) => page,
                Err(GmailAdapterErrorV1::ProviderStatus(404)) => return Err(GmailHistorySyncError::Expired),
                Err(_) => return Err(GmailHistorySyncError::Runtime(MailBootstrapError::Provider)),
            };
            let checkpoint_history_id = valid_gmail_history_id(page.history_id.as_deref()).ok_or(GmailHistorySyncError::Runtime(MailBootstrapError::Provider))?;
            let message_ids = history_message_ids(&page);
            let (records, _) = self.gmail_message_records(connection_id, token, client, message_ids.clone().into_iter(), observed_at_unix_seconds, observed_at_nanos).await.map_err(GmailHistorySyncError::Runtime)?;
            observed_messages = observed_messages.saturating_add(message_ids.len());
            let next_page_token = page.next_page_token;
            let next_checkpoint = if next_page_token.is_some() { start_history_id } else { checkpoint_history_id };
            self.durable.enqueue_communications_outbox_and_store_gmail_history_checkpoint(&records, connection_id, next_checkpoint, next_page_token.as_deref(), observed_at_unix_seconds).await.map_err(|_| GmailHistorySyncError::Runtime(MailBootstrapError::Persistence))?;
            let has_next_page = next_page_token.is_some();
            page_token = next_page_token;
            if !has_next_page {
                break;
            }
        }
        Ok(observed_messages)
    }

    async fn gmail_message_records(
        &mut self,
        connection_id: &str,
        token: &str,
        client: &GmailApiClientV1,
        message_ids: impl Iterator<Item = String>,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<(Vec<hermes_events_protocol::delivery::OutboxRecordV1>, Option<String>), MailBootstrapError> {
        let mut records = Vec::new();
        let mut observed_history_id = None;
        for message_id in message_ids {
                let raw = client.fetch_raw_message(token, &message_id).await.map_err(|_| MailBootstrapError::Provider)?;
                let bytes = raw.raw.as_deref().ok_or(MailBootstrapError::Provider).and_then(|value| decode_raw_rfc822(value).map_err(|_| MailBootstrapError::Provider))?;
                observed_history_id = newer_gmail_history_id(observed_history_id.as_deref(), raw.history_id.as_deref()).map(str::to_owned);
                let provider_record_id = raw.id.unwrap_or(message_id);
                let observation = self.draft_inbound_body_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailGmail,
                        connection_id,
                        &provider_record_id,
                        None,
                    ),
                    ProviderProvenanceV1::MailGmail,
                    connection_id,
                    format!("{connection_id}:{provider_record_id}"),
                    direct_plain_text_body(&bytes),
                )?;
                records.push(build_observation_outbox_record_v1(
                    &observation,
                    &ObservationEnvelopeContextV1 {
                        runtime_instance_id: self.runtime_instance_id.clone(),
                        runtime_generation: self.runtime_generation,
                        module_id: "mail-runtime".to_owned(),
                        recorded_at_unix_seconds: observed_at_unix_seconds,
                        recorded_at_nanos: observed_at_nanos,
                    },
                ).map_err(|_| MailBootstrapError::Admission)?);
                for attachment in attachment_metadata(&bytes) {
                    let source_id = format!("{connection_id}:{provider_record_id}");
                    let media_id = format!("{}:{}", provider_record_id, attachment.part_id);
                    let disposition = match attachment.disposition {
                        Rfc822AttachmentDispositionV1::Attachment => AttachmentDispositionV1::Attachment,
                        Rfc822AttachmentDispositionV1::Inline => AttachmentDispositionV1::Inline,
                    };
                    let observation = draft_attachment_ingress_observation(
                        &inbound_observation_id(
                            ProviderProvenanceV1::MailGmail,
                            connection_id,
                            &provider_record_id,
                            Some(attachment.part_id),
                        ),
                        ProviderProvenanceV1::MailGmail,
                        connection_id,
                        source_id,
                        media_id,
                        attachment.filename,
                        attachment.media_type,
                        attachment.declared_bytes,
                        disposition,
                    ).map_err(|_| MailBootstrapError::Provider)?;
                    records.push(build_observation_outbox_record_v1(
                        &observation,
                        &ObservationEnvelopeContextV1 {
                            runtime_instance_id: self.runtime_instance_id.clone(),
                            runtime_generation: self.runtime_generation,
                            module_id: "mail-runtime".to_owned(),
                            recorded_at_unix_seconds: observed_at_unix_seconds,
                            recorded_at_nanos: observed_at_nanos,
                        },
                    ).map_err(|_| MailBootstrapError::Admission)?);
                }
            }
        Ok((records, observed_history_id))
    }

    fn draft_inbound_body_observation(
        &mut self,
        operation_id: &str,
        provider: ProviderProvenanceV1,
        connection_id: &str,
        source_id: String,
        plaintext: Option<Vec<u8>>,
    ) -> Result<CommunicationObservationDraft, MailBootstrapError> {
        let Some(plaintext) = plaintext else {
            return unavailable_body_observation(operation_id, provider, connection_id, source_id, BodyAdmissionFailureV1::PolicyRejected);
        };
        match self.admit_plain_text_body(&plaintext) {
            Ok(receipt) => with_admitted_body_blob(
                draft_ingress_observation_with_body(operation_id, provider, connection_id, source_id, BodyAvailabilityV1::AdmittedBlob)
                    .map_err(|_| MailBootstrapError::Provider)?,
                receipt,
            ).map_err(|_| MailBootstrapError::Provider),
            Err(failure) => unavailable_body_observation(operation_id, provider, connection_id, source_id, failure),
        }
    }

    fn admit_plain_text_body(&mut self, plaintext: &[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
        if plaintext.is_empty() || plaintext.len() > hermes_mail_api::MAX_PLAIN_TEXT_BYTES {
            return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        if reference_id.iter().all(|byte| *byte == 0) { return Err(BodyAdmissionFailureV1::SourceUnavailable); }
        let sha256: [u8; 32] = Sha256::digest(plaintext).into();
        self.control_channel.set_nonblocking(false).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let session = request_managed_blob_session(
            &mut self.control_channel,
            "blob.content",
            BlobDataOperationV1::BlobDataOperationWriteV1,
            &reference_id,
            u64::try_from(plaintext.len()).map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            1,
            Some(&sha256),
        );
        let restored = self.control_channel.set_nonblocking(true);
        let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
        restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let custody_transfer_source_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| client.write(session.grant, session.channel_binding, plaintext.to_vec()))
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        Ok(BodyBlobReceiptV1 {
            blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            sha256,
            custody_transfer_source_proof,
        })
    }
}

fn valid_gmail_history_id(value: Option<&str>) -> Option<&str> {
    value.filter(|history_id| !history_id.is_empty() && history_id.bytes().all(|byte| byte.is_ascii_digit()))
}

fn newer_gmail_history_id<'a>(current: Option<&'a str>, candidate: Option<&'a str>) -> Option<&'a str> {
    match (valid_gmail_history_id(current), valid_gmail_history_id(candidate)) {
        (None, value) | (value, None) => value,
        (Some(current), Some(candidate)) if candidate.len() > current.len() || (candidate.len() == current.len() && candidate > current) => Some(candidate),
        (Some(current), Some(_)) => Some(current),
    }
}

#[cfg(test)]
mod gmail_history_checkpoint_tests {
    use super::{newer_gmail_history_id, valid_gmail_history_id};

    #[test]
    fn checkpoint_accepts_only_numeric_ids_and_never_regresses() {
        assert_eq!(valid_gmail_history_id(Some("")), None);
        assert_eq!(valid_gmail_history_id(Some("history-12")), None);
        assert_eq!(valid_gmail_history_id(Some("12")), Some("12"));
        assert_eq!(newer_gmail_history_id(Some("12"), Some("9")), Some("12"));
        assert_eq!(newer_gmail_history_id(Some("12"), Some("100")), Some("100"));
        assert_eq!(newer_gmail_history_id(None, Some("100")), Some("100"));
    }
}

fn unavailable_body_observation(
    operation_id: &str,
    provider: ProviderProvenanceV1,
    connection_id: &str,
    source_id: String,
    failure: BodyAdmissionFailureV1,
) -> Result<CommunicationObservationDraft, MailBootstrapError> {
    with_body_admission_failure(
        draft_ingress_observation_with_body(operation_id, provider, connection_id, source_id, BodyAvailabilityV1::Unavailable)
            .map_err(|_| MailBootstrapError::Provider)?,
        failure,
    ).map_err(|_| MailBootstrapError::Provider)
}

fn inbound_observation_id(
    provider: ProviderProvenanceV1,
    connection_id: &str,
    provider_record_id: &str,
    attachment_part_id: Option<u16>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.mail.inbound-observation.v1\0");
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(connection_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_record_id.as_bytes());
    hasher.update(b"\0");
    if let Some(part_id) = attachment_part_id {
        hasher.update(part_id.to_be_bytes());
    }
    format!("mail-inbound:{}", hex_digest(&hasher.finalize()))
}

fn hex_digest(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_identity_is_stable_across_sync_operations_and_distinguishes_parts() {
        let message = inbound_observation_id(
            ProviderProvenanceV1::MailImap,
            "account-1",
            "uid-42",
            None,
        );

        assert_eq!(
            message,
            inbound_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                None,
            ),
        );
        assert_ne!(
            message,
            inbound_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                Some(1),
            ),
        );
    }
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn credential_revision(
    admission: &MailRuntimeAdmission,
    purpose: MailCredentialPurpose,
) -> Result<Option<u64>, MailBootstrapError> {
    let revision = match purpose {
        MailCredentialPurpose::ImapPassword => admission.credential_revisions.imap_password,
        MailCredentialPurpose::GmailAccessToken => admission.credential_revisions.gmail_access_token,
        MailCredentialPurpose::SmtpPassword => admission.credential_revisions.smtp_password,
    };
    revision
        .is_none_or(|value| value != 0)
        .then_some(revision)
        .ok_or(MailBootstrapError::Admission)
}

fn provider_credential_context(
    admission: &MailRuntimeAdmission,
    configuration: &ManagedStorageRuntimeConfigurationV1,
) -> Result<ManagedProviderCredentialContextV1, MailBootstrapError> {
    let vault_public_key_x25519 = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| MailBootstrapError::Admission)?;
    if configuration.vault_runtime_generation != admission.vault_runtime_generation {
        return Err(MailBootstrapError::Admission);
    }
    Ok(ManagedProviderCredentialContextV1 {
        vault_instance_id: configuration.vault_instance_id.clone(),
        vault_runtime_generation: configuration.vault_runtime_generation,
        vault_public_key_x25519,
        logical_owner_id: admission.logical_owner_id.clone(),
        registration_id: admission.module_registration_id.clone(),
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
        grant_epoch: admission.grant_epoch,
    })
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &MailRuntimeAdmission,
) -> Result<StorageBindingV1, MailBootstrapError> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(MailBootstrapError::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.module_registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections).map_err(|_| MailBootstrapError::Storage)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailBootstrapError::Storage)?,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    StorageBindingV1::new(identity, fences, access).map_err(|_| MailBootstrapError::Storage)
}

fn map_provider_credential_error(error: ManagedProviderCredentialErrorV1) -> MailBootstrapError {
    match error {
        ManagedProviderCredentialErrorV1::InvalidContext => MailBootstrapError::Admission,
        ManagedProviderCredentialErrorV1::Rejected | ManagedProviderCredentialErrorV1::Unavailable => MailBootstrapError::Credential,
    }
}

fn current_unix_seconds() -> Result<i64, MailBootstrapError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MailBootstrapError::Provider)
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).map_err(|_| MailBootstrapError::Provider))
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), MailBootstrapError> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(MailBootstrapError::Control);
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| MailBootstrapError::Control)?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    channel.write_all(&prefix).and_then(|_| channel.write_all(bytes)).and_then(|_| channel.flush())
        .map_err(|_| MailBootstrapError::Control)
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, MailBootstrapError> {
    let length = usize::try_from(read_varint(channel)?).map_err(|_| MailBootstrapError::Control)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(MailBootstrapError::Control);
    }
    let mut bytes = vec![0_u8; length];
    channel.read_exact(&mut bytes).map_err(|_| MailBootstrapError::Control)?;
    Ok(bytes)
}

fn peek_complete_frame(channel: &UnixStream) -> Result<Option<Vec<u8>>, MailBootstrapError> {
    let mut header = [0_u8; 5];
    let length = unsafe { libc::recv(channel.as_raw_fd(), header.as_mut_ptr().cast(), header.len(), libc::MSG_PEEK) };
    if length < 0 {
        return if std::io::Error::last_os_error().kind() == std::io::ErrorKind::WouldBlock { Ok(None) } else { Err(MailBootstrapError::Control) };
    }
    if length == 0 { return Err(MailBootstrapError::Control); }
    let (payload_length, prefix_length) = decode_peeked_length(&header[..usize::try_from(length).map_err(|_| MailBootstrapError::Control)?])?;
    if payload_length == 0 || payload_length > MAX_FRAME_BYTES { return Err(MailBootstrapError::Control); }
    let mut framed = vec![0_u8; prefix_length + payload_length];
    let received = unsafe { libc::recv(channel.as_raw_fd(), framed.as_mut_ptr().cast(), framed.len(), libc::MSG_PEEK) };
    if received < 0 { return Err(MailBootstrapError::Control); }
    if usize::try_from(received).map_err(|_| MailBootstrapError::Control)? < framed.len() { return Ok(None); }
    Ok(Some(framed[prefix_length..].to_vec()))
}

fn decode_peeked_length(bytes: &[u8]) -> Result<(usize, usize), MailBootstrapError> {
    let mut value = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        value |= usize::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 { return Ok((value, index + 1)); }
    }
    Err(MailBootstrapError::Control)
}

fn read_varint(channel: &mut UnixStream) -> Result<u64, MailBootstrapError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel.read_exact(&mut byte).map_err(|_| MailBootstrapError::Control)?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(MailBootstrapError::Control)
}
