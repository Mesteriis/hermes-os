//! Kernel-admitted Mail runtime bootstrap. No CLI, provider, or domain fallback exists here.

use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_blob_client::{
    BlobDataClient, ManagedBlobSessionRequestV1, request_managed_blob_session_v2,
};
use hermes_communications_ingress::{
    AttachmentBlobAdmissionFactV1, AttachmentBlobAdmissionTransitionV1,
    AttachmentBlobExpectedStateV1, ObservationEnvelopeContextV1,
    build_attachment_blob_admission_outbox_record_v1, build_observation_outbox_record_v1,
};
use hermes_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1, ManagedProviderCredentialRequestV1,
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use hermes_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
        RejectManagedControlRequestsV2,
    },
    validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use hermes_vault_protocol::SecretClassV1;
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::MailRuntimeAdmission;
use crate::admission::{
    MAIL_BLOB_CAPABILITY_ID, MAIL_CREDENTIAL_LEASE_TTL_SECONDS, MAIL_MODULE_ID,
};
use crate::attachment_anchor_mapping::{
    MailAttachmentAnchorMappingErrorV1, consume_next_attachment_anchor_recorded_v1,
};
use crate::communications_outbox::{
    MailCommunicationsOutboxRelayError, relay_communications_outbox_once,
};
use hermes_communications_ingress::{
    AttachmentDispositionV1, BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1,
    CommunicationObservationDraft, ProviderProvenanceV1, with_admitted_body_blob,
    with_body_admission_failure,
};
use hermes_mail_api::{
    MailCredentialPurpose, MailInboundTransportV1, MailSendMailRequestV1, OutgoingMailV1,
    valid_account_configuration, valid_port,
};
use hermes_mail_core::rfc822::{
    AttachmentDispositionV1 as Rfc822AttachmentDispositionV1, attachment_metadata,
    direct_plain_text_body,
};
use hermes_mail_core::{
    bounded_window, compose_rfc822, draft_attachment_ingress_observation,
    draft_delivery_observation, draft_ingress_observation_with_body, validate_sync_request,
};
use hermes_mail_gmail::{
    GmailAdapterErrorV1, GmailApiClientV1, GmailListMessagesRequestV1, decode_raw_rfc822,
    history_message_ids,
};
use hermes_mail_persistence::MailDurablePersistence;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct MailAdmittedRuntime {
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub durable: MailDurablePersistence,
    inbound_credential: MailInboundCredentialV1,
    smtp_password: Option<Zeroizing<Vec<u8>>>,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    attachment_anchor_subscribe_permit: Option<RuntimeSubscribePermitV1>,
    attachment_blob_admission_publish_permitted: bool,
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

struct ImapInboxSyncRequestV1<'a> {
    connection_id: &'a str,
    operation_id: &'a str,
    host: &'a str,
    port: u16,
    username: &'a str,
    window: u32,
    windows: u32,
}

struct GmailHistorySyncRequestV1<'a> {
    connection_id: &'a str,
    token: &'a str,
    client: &'a GmailApiClientV1,
    start_history_id: &'a str,
    page_token: Option<String>,
    windows: u32,
    observed_at_unix_seconds: i64,
    observed_at_nanos: i32,
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
    AttachmentAnchorMapping,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    control_channel: UnixStream,
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
    let mut control_channel = ManagedControlChannelV2::new(control_channel);
    let identity = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| MailBootstrapError::Control)?;
    let registration_id = identity.registration_id;
    let runtime_generation = identity.runtime_generation;
    let grant_epoch = identity.grant_epoch;
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(MailBootstrapError::Admission);
    }

    let provider_context = provider_credential_context(admission, &storage_configuration)?;
    let (inbound_credential, smtp_password) = {
        let mut provider_credentials = ManagedProviderCredentialClientV2::new(&mut control_channel);
        let mut dispatcher = RejectManagedControlRequestsV2;
        let inbound_credential = match &admission.account.inbound {
            MailInboundTransportV1::Imap(_) => {
                let revision = credential_revision(admission, MailCredentialPurpose::ImapPassword)?
                    .ok_or(MailBootstrapError::Admission)?;
                MailInboundCredentialV1::ImapPassword(
                    provider_credentials
                        .resolve(
                            &mut dispatcher,
                            &provider_context,
                            ManagedProviderCredentialRequestV1 {
                                configuration_instance_id: &admission.configuration_instance_id,
                                purpose_id: MailCredentialPurpose::ImapPassword.as_str(),
                                credential_revision: revision,
                                ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                                secret_class: SecretClassV1::ProviderCredential,
                            },
                        )
                        .map_err(map_provider_credential_error)?,
                )
            }
            MailInboundTransportV1::Gmail(_) => {
                let revision =
                    credential_revision(admission, MailCredentialPurpose::GmailAccessToken)?
                        .ok_or(MailBootstrapError::Admission)?;
                MailInboundCredentialV1::GmailAccessToken(
                    provider_credentials
                        .resolve(
                            &mut dispatcher,
                            &provider_context,
                            ManagedProviderCredentialRequestV1 {
                                configuration_instance_id: &admission.configuration_instance_id,
                                purpose_id: MailCredentialPurpose::GmailAccessToken.as_str(),
                                credential_revision: revision,
                                ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                                secret_class: SecretClassV1::ProviderCredential,
                            },
                        )
                        .map_err(map_provider_credential_error)?,
                )
            }
        };
        let smtp_password =
            match credential_revision(admission, MailCredentialPurpose::SmtpPassword)? {
                Some(revision) => Some(
                    provider_credentials
                        .resolve(
                            &mut dispatcher,
                            &provider_context,
                            ManagedProviderCredentialRequestV1 {
                                configuration_instance_id: &admission.configuration_instance_id,
                                purpose_id: MailCredentialPurpose::SmtpPassword.as_str(),
                                credential_revision: revision,
                                ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                                secret_class: SecretClassV1::ProviderCredential,
                            },
                        )
                        .map_err(map_provider_credential_error)?,
                ),
                None => None,
            };
        (inbound_credential, smtp_password)
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
        InheritedKernelVaultRouteV2::new(control_channel),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|error| {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_mail_storage_credential_issue_error={error:?}");
            }
            MailBootstrapError::Credential
        })?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|error| {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_mail_storage_credential_resolve_error={error:?}");
            }
            MailBootstrapError::Credential
        })?;
    let mut control_channel = storage_leases.into_route_port().into_channel();
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
    let event_access = request_managed_runtime_event_access_v2(
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
    let attachment_anchor_subscribe_permit = bind_attachment_anchor_subscribe_permit(
        event_access
            .subscribe_permits(
                &admission.module_registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| MailBootstrapError::EventHub)?,
    )?;
    let attachment_blob_admission_publish_permitted =
        attachment_blob_admission_publish_permitted(&event_publish_permit)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| MailBootstrapError::EventHub)?;
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id,
            runtime_generation,
            grant_epoch,
        })
        .map_err(|_| MailBootstrapError::Control)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .and_then(|_| control_channel.inner_mut().set_nonblocking(true))
        .map_err(|_| MailBootstrapError::Control)?;
    Ok(MailAdmittedRuntime {
        control_channel,
        durable,
        inbound_credential,
        smtp_password,
        event_connection,
        event_publish_permit,
        attachment_anchor_subscribe_permit,
        attachment_blob_admission_publish_permitted,
        account: admission.account.clone(),
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
    })
}

impl MailAdmittedRuntime {
    pub async fn try_consume_attachment_anchor_handoff(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailBootstrapError> {
        let Some(permit) = &self.attachment_anchor_subscribe_permit else {
            return Ok(false);
        };
        match tokio::time::timeout(
            Duration::from_millis(25),
            consume_next_attachment_anchor_recorded_v1(
                &self.durable,
                &self.event_connection,
                permit,
                consumed_at_unix_seconds,
            ),
        )
        .await
        {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(MailAttachmentAnchorMappingErrorV1::Unavailable)) => Ok(false),
            Ok(Err(error)) => Err(map_attachment_anchor_mapping_error(error)),
            Err(_) => Ok(false),
        }
    }

    pub async fn try_handle_client_delivery(&mut self) -> Result<bool, MailBootstrapError> {
        let Some((correlation_id, control_request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| MailBootstrapError::Control)?
        else {
            return Ok(false);
        };
        let request = match control_request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => request,
                _ => {
                    write_control_error(
                        &mut self.control_channel,
                        correlation_id,
                        "managed_runtime_control_invalid_client_delivery",
                    )?;
                    return Ok(true);
                }
            },
            _ => {
                write_control_error(
                    &mut self.control_channel,
                    correlation_id,
                    "managed_runtime_control_unexpected_request",
                )?;
                return Ok(true);
            }
        };
        let payload = crate::client_port::handle_client_request(self, &request.encode_to_vec())
            .await
            .map_err(|_| MailBootstrapError::Provider)?;
        let response = ModuleClientResponseV1::decode(payload.as_slice())
            .map_err(|_| MailBootstrapError::Provider)?;
        validate_module_client_response_v1(&response).map_err(|_| MailBootstrapError::Provider)?;
        write_client_delivery_response(&mut self.control_channel, correlation_id, response)?;
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
            MailInboundTransportV1::Imap(_) => {
                self.send_mail_via_smtp(
                    account
                        .smtp_endpoint
                        .as_ref()
                        .ok_or(MailBootstrapError::Admission)?,
                    &message,
                )
                .await
            }
            MailInboundTransportV1::Gmail(configuration) => {
                self.send_mail_via_gmail(
                    &configuration.user_id,
                    &configuration.from_address,
                    &message,
                )
                .await
            }
        }
    }

    async fn send_mail_via_smtp(
        &mut self,
        endpoint: &hermes_mail_api::SmtpEndpointV1,
        message: &OutgoingMailV1,
    ) -> Result<u16, MailBootstrapError> {
        let password = self
            .smtp_password
            .as_deref()
            .ok_or(MailBootstrapError::Credential)?;
        let password = std::str::from_utf8(password).map_err(|_| MailBootstrapError::Credential)?;
        self.send_mail(
            message,
            &endpoint.from_address,
            ProviderProvenanceV1::MailSmtp,
            |rfc822_message| async move {
                hermes_mail_smtp::send_implicit_tls(endpoint, message, password, &rfc822_message)
                    .await
                    .map(|receipt| receipt.response_code)
            },
        )
        .await
    }

    async fn send_mail_via_gmail(
        &self,
        user_id: &str,
        from_address: &str,
        message: &OutgoingMailV1,
    ) -> Result<u16, MailBootstrapError> {
        let MailInboundCredentialV1::GmailAccessToken(access_token) = &self.inbound_credential
        else {
            return Err(MailBootstrapError::Credential);
        };
        let access_token =
            std::str::from_utf8(access_token).map_err(|_| MailBootstrapError::Credential)?;
        self.send_mail(
            message,
            from_address,
            ProviderProvenanceV1::MailGmail,
            |rfc822_message| async move {
                let client = hermes_mail_gmail::GmailApiClientV1::new(user_id)
                    .map_err(|_| hermes_mail_gmail::GmailAdapterErrorV1::Transport)?;
                client
                    .send_raw_message(
                        access_token,
                        rfc822_message.as_bytes(),
                        Some(&message.provider_conversation_id),
                    )
                    .await
                    .map(|_| 200)
            },
        )
        .await
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
        let rfc822_message =
            compose_rfc822(from_address, message).map_err(|_| MailBootstrapError::Admission)?;
        let rfc822_sha256: [u8; 32] = Sha256::digest(rfc822_message.as_bytes()).into();
        let attempted_at = current_unix_seconds()?;
        let started = self
            .durable
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
        let observation = draft_delivery_observation(provider, message)
            .map_err(|_| MailBootstrapError::Admission)?;
        let record = build_observation_outbox_record_v1(
            &observation,
            &observation_context(
                &self.runtime_instance_id,
                self.runtime_generation,
                completed_at,
                0,
            ),
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
            MailInboundTransportV1::Imap(configuration) => {
                self.sync_inbox(ImapInboxSyncRequestV1 {
                    connection_id: &account.connection_id,
                    operation_id,
                    host: &configuration.host,
                    port: configuration.port,
                    username: &configuration.username,
                    window: account.sync_window,
                    windows: account.sync_windows,
                })
                .await
            }
            MailInboundTransportV1::Gmail(configuration) => {
                self.sync_gmail_inbox(
                    &account.connection_id,
                    operation_id,
                    &configuration.user_id,
                    account.sync_window,
                    account.sync_windows,
                )
                .await
            }
        }
    }

    async fn sync_inbox(
        &mut self,
        request: ImapInboxSyncRequestV1<'_>,
    ) -> Result<usize, MailBootstrapError> {
        let ImapInboxSyncRequestV1 {
            connection_id,
            operation_id,
            host,
            port,
            username,
            window,
            windows,
        } = request;
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
        let password =
            std::str::from_utf8(&password).map_err(|_| MailBootstrapError::Credential)?;
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
        let observed_at_unix_seconds =
            i64::try_from(observed_at.as_secs()).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos =
            i32::try_from(observed_at.subsec_nanos()).map_err(|_| MailBootstrapError::Provider)?;
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
            )?;
            let record = build_observation_outbox_record_v1(
                &observation,
                &observation_context(
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                ),
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
                    hermes_mail_imap::ImapAttachmentDisposition::Attachment => {
                        AttachmentDispositionV1::Attachment
                    }
                    hermes_mail_imap::ImapAttachmentDisposition::Inline => {
                        AttachmentDispositionV1::Inline
                    }
                };
                let observation = draft_attachment_ingress_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailImap,
                        connection_id,
                        &message.uid.to_string(),
                        Some(attachment.part_id),
                    ),
                    hermes_mail_core::MailAttachmentIngressRequestV1 {
                        provider: ProviderProvenanceV1::MailImap,
                        account_id: connection_id.to_owned(),
                        message_source_id: source_id,
                        media_id,
                        filename: attachment.filename.clone(),
                        media_type: attachment.media_type.clone(),
                        declared_bytes: attachment.declared_bytes,
                        disposition,
                    },
                )
                .map_err(|_| MailBootstrapError::Provider)?;
                let record = build_observation_outbox_record_v1(
                    &observation,
                    &observation_context(
                        &self.runtime_instance_id,
                        self.runtime_generation,
                        observed_at_unix_seconds,
                        observed_at_nanos,
                    ),
                )
                .map_err(|_| MailBootstrapError::Admission)?;
                self.durable
                    .enqueue_communications_outbox(&record, observed_at_unix_seconds)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                self.try_admit_imap_attachment_blob(
                    *record.message_id(),
                    attachment.bytes(),
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .await?;
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
        let max_results =
            u16::try_from(plan.window.min(500)).map_err(|_| MailBootstrapError::Admission)?;
        let client = GmailApiClientV1::new(user_id).map_err(|_| MailBootstrapError::Admission)?;
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_unix_seconds =
            i64::try_from(observed_at.as_secs()).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos =
            i32::try_from(observed_at.subsec_nanos()).map_err(|_| MailBootstrapError::Provider)?;
        if let Some((start_history_id, page_token)) = self
            .durable
            .gmail_history_checkpoint(connection_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
        {
            match self
                .sync_gmail_history_pages(GmailHistorySyncRequestV1 {
                    connection_id,
                    token,
                    client: &client,
                    start_history_id: &start_history_id,
                    page_token,
                    windows: plan.windows,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                })
                .await
            {
                Ok(observed_messages) => return Ok(observed_messages),
                Err(GmailHistorySyncError::Expired) => self
                    .durable
                    .clear_gmail_history_checkpoint(connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?,
                Err(GmailHistorySyncError::Runtime(error)) => return Err(error),
            }
        }
        let mut observed_messages = 0_usize;
        let (mut page_token, mut observed_history_id) = self
            .durable
            .gmail_sync_progress(connection_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
            .map(|(page_token, observed_history_id)| (Some(page_token), observed_history_id))
            .unwrap_or((None, None));
        for _ in 0..plan.windows {
            let page = client
                .list_messages(
                    token,
                    &GmailListMessagesRequestV1 {
                        max_results,
                        page_token: page_token.clone(),
                        query: None,
                        label_ids: Vec::new(),
                    },
                )
                .await
                .map_err(|_| MailBootstrapError::Provider)?;
            let next_page_token = page.next_page_token.clone();
            let listed_messages = page.messages;
            let page_message_count = listed_messages.len();
            let (records, page_history_id) = self
                .gmail_message_records(
                    connection_id,
                    token,
                    &client,
                    listed_messages.into_iter().map(|message| message.id),
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .await?;
            observed_messages = observed_messages.saturating_add(page_message_count);
            observed_history_id =
                newer_gmail_history_id(observed_history_id.as_deref(), page_history_id.as_deref())
                    .map(str::to_owned);
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
        request: GmailHistorySyncRequestV1<'_>,
    ) -> Result<usize, GmailHistorySyncError> {
        let GmailHistorySyncRequestV1 {
            connection_id,
            token,
            client,
            start_history_id,
            mut page_token,
            windows,
            observed_at_unix_seconds,
            observed_at_nanos,
        } = request;
        let mut observed_messages = 0_usize;
        for _ in 0..windows {
            let page = match client
                .list_history(token, start_history_id, page_token.as_deref())
                .await
            {
                Ok(page) => page,
                Err(GmailAdapterErrorV1::ProviderStatus(404)) => {
                    return Err(GmailHistorySyncError::Expired);
                }
                Err(_) => return Err(GmailHistorySyncError::Runtime(MailBootstrapError::Provider)),
            };
            let checkpoint_history_id = valid_gmail_history_id(page.history_id.as_deref())
                .ok_or(GmailHistorySyncError::Runtime(MailBootstrapError::Provider))?;
            let message_ids = history_message_ids(&page);
            let (records, _) = self
                .gmail_message_records(
                    connection_id,
                    token,
                    client,
                    message_ids.clone().into_iter(),
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .await
                .map_err(GmailHistorySyncError::Runtime)?;
            observed_messages = observed_messages.saturating_add(message_ids.len());
            let next_page_token = page.next_page_token;
            let next_checkpoint = if next_page_token.is_some() {
                start_history_id
            } else {
                checkpoint_history_id
            };
            self.durable
                .enqueue_communications_outbox_and_store_gmail_history_checkpoint(
                    &records,
                    connection_id,
                    next_checkpoint,
                    next_page_token.as_deref(),
                    observed_at_unix_seconds,
                )
                .await
                .map_err(|_| GmailHistorySyncError::Runtime(MailBootstrapError::Persistence))?;
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
    ) -> Result<
        (
            Vec<hermes_events_protocol::delivery::OutboxRecordV1>,
            Option<String>,
        ),
        MailBootstrapError,
    > {
        let mut records = Vec::new();
        let mut observed_history_id = None;
        for message_id in message_ids {
            let raw = client
                .fetch_raw_message(token, &message_id)
                .await
                .map_err(|_| MailBootstrapError::Provider)?;
            let bytes = raw
                .raw
                .as_deref()
                .ok_or(MailBootstrapError::Provider)
                .and_then(|value| {
                    decode_raw_rfc822(value).map_err(|_| MailBootstrapError::Provider)
                })?;
            observed_history_id =
                newer_gmail_history_id(observed_history_id.as_deref(), raw.history_id.as_deref())
                    .map(str::to_owned);
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
            records.push(
                build_observation_outbox_record_v1(
                    &observation,
                    &observation_context(
                        &self.runtime_instance_id,
                        self.runtime_generation,
                        observed_at_unix_seconds,
                        observed_at_nanos,
                    ),
                )
                .map_err(|_| MailBootstrapError::Admission)?,
            );
            for attachment in attachment_metadata(&bytes) {
                let source_id = format!("{connection_id}:{provider_record_id}");
                let media_id = format!("{}:{}", provider_record_id, attachment.part_id);
                let disposition = match attachment.disposition {
                    Rfc822AttachmentDispositionV1::Attachment => {
                        AttachmentDispositionV1::Attachment
                    }
                    Rfc822AttachmentDispositionV1::Inline => AttachmentDispositionV1::Inline,
                };
                let observation = draft_attachment_ingress_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailGmail,
                        connection_id,
                        &provider_record_id,
                        Some(attachment.part_id),
                    ),
                    hermes_mail_core::MailAttachmentIngressRequestV1 {
                        provider: ProviderProvenanceV1::MailGmail,
                        account_id: connection_id.to_owned(),
                        message_source_id: source_id,
                        media_id,
                        filename: attachment.filename,
                        media_type: attachment.media_type,
                        declared_bytes: attachment.declared_bytes,
                        disposition,
                    },
                )
                .map_err(|_| MailBootstrapError::Provider)?;
                records.push(
                    build_observation_outbox_record_v1(
                        &observation,
                        &observation_context(
                            &self.runtime_instance_id,
                            self.runtime_generation,
                            observed_at_unix_seconds,
                            observed_at_nanos,
                        ),
                    )
                    .map_err(|_| MailBootstrapError::Admission)?,
                );
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
            return unavailable_body_observation(
                operation_id,
                provider,
                connection_id,
                source_id,
                BodyAdmissionFailureV1::PolicyRejected,
            );
        };
        match self.admit_plain_text_body(&plaintext) {
            Ok(receipt) => with_admitted_body_blob(
                draft_ingress_observation_with_body(
                    operation_id,
                    provider,
                    connection_id,
                    source_id,
                    BodyAvailabilityV1::AdmittedBlob,
                )
                .map_err(|_| MailBootstrapError::Provider)?,
                receipt,
            )
            .map_err(|_| MailBootstrapError::Provider),
            Err(failure) => unavailable_body_observation(
                operation_id,
                provider,
                connection_id,
                source_id,
                failure,
            ),
        }
    }

    fn admit_plain_text_body(
        &mut self,
        plaintext: &[u8],
    ) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
        if plaintext.is_empty() || plaintext.len() > hermes_mail_api::MAX_PLAIN_TEXT_BYTES {
            return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id)
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err(BodyAdmissionFailureV1::SourceUnavailable);
        }
        let sha256: [u8; 32] = Sha256::digest(plaintext).into();
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let mut dispatcher = MailBusyControlDispatcher;
        let session = request_managed_blob_session_v2(
            &mut self.control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: MAIL_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: u64::try_from(plaintext.len())
                    .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
                backup_class: 1,
                receipt_sha256: Some(&sha256),
            },
        );
        let restored = self.control_channel.inner_mut().set_nonblocking(true);
        let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
        restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let custody_transfer_source_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.write(session.grant, session.channel_binding, plaintext.to_vec())
            })
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        Ok(BodyBlobReceiptV1 {
            blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len())
                .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            sha256,
            custody_transfer_source_proof,
        })
    }

    async fn try_admit_imap_attachment_blob(
        &mut self,
        source_observation_id: [u8; 16],
        bytes: &[u8],
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<(), MailBootstrapError> {
        if !self.attachment_blob_admission_publish_permitted {
            return Ok(());
        }
        let Some(mapping) = self
            .durable
            .attachment_anchor_mapping(source_observation_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
        else {
            return Ok(());
        };
        let context = observation_context(
            &self.runtime_instance_id,
            self.runtime_generation,
            observed_at_unix_seconds,
            observed_at_nanos,
        );
        let requested = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: mapping.attachment_anchor_id,
                source_observation_id,
                correlation_id: mapping.correlation_id,
                media_cursor_sha256: mapping.media_cursor_sha256,
                expected_state: AttachmentBlobExpectedStateV1::DescriptorOnly,
                transition: AttachmentBlobAdmissionTransitionV1::Requested,
                observed_at_unix_seconds,
                blob_reference_binding_sha256: None,
            },
            &context,
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        let outcome = self
            .durable
            .begin_attachment_blob_admission(
                source_observation_id,
                mapping.attachment_anchor_id,
                &requested,
                observed_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        if !matches!(
            outcome,
            hermes_mail_persistence::MailAttachmentBlobAdmissionStartOutcomeV1::Started
        ) {
            return Ok(());
        }
        let terminal = match self.write_attachment_blob(bytes) {
            Ok(binding) => (
                2,
                AttachmentBlobAdmissionTransitionV1::Admitted,
                Some(binding),
            ),
            Err(_) => (3, AttachmentBlobAdmissionTransitionV1::Rejected, None),
        };
        let terminal_record = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: mapping.attachment_anchor_id,
                source_observation_id,
                correlation_id: mapping.correlation_id,
                media_cursor_sha256: mapping.media_cursor_sha256,
                expected_state: AttachmentBlobExpectedStateV1::BlobPending,
                transition: terminal.1,
                observed_at_unix_seconds,
                blob_reference_binding_sha256: terminal.2,
            },
            &context,
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .complete_attachment_blob_admission(
                source_observation_id,
                mapping.attachment_anchor_id,
                terminal.0,
                &terminal_record,
                observed_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        Ok(())
    }

    fn write_attachment_blob(&mut self, bytes: &[u8]) -> Result<[u8; 32], MailBootstrapError> {
        if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
            return Err(MailBootstrapError::Admission);
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id).map_err(|_| MailBootstrapError::Control)?;
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err(MailBootstrapError::Control);
        }
        let receipt_sha256: [u8; 32] = Sha256::digest(bytes).into();
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| MailBootstrapError::Control)?;
        let mut dispatcher = MailBusyControlDispatcher;
        let session = request_managed_blob_session_v2(
            &mut self.control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: MAIL_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: u64::try_from(bytes.len())
                    .map_err(|_| MailBootstrapError::Admission)?,
                backup_class: 1,
                receipt_sha256: Some(&receipt_sha256),
            },
        );
        let restored = self.control_channel.inner_mut().set_nonblocking(true);
        let session = session.map_err(|_| MailBootstrapError::Control)?;
        restored.map_err(|_| MailBootstrapError::Control)?;
        if session.custody_transfer_source_proof.is_empty() {
            return Err(MailBootstrapError::Control);
        }
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
            .map_err(|_| MailBootstrapError::Control)?;
        Ok(Sha256::digest(session.custody_transfer_source_proof).into())
    }
}

struct MailBusyControlDispatcher;

impl ManagedControlRequestDispatcherV2<UnixStream> for MailBusyControlDispatcher {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "RUNTIME_BUSY".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), MailBootstrapError> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ClientDelivery(
                    ManagedRuntimeClientDeliveryResponseV1 {
                        response: Some(response),
                    },
                )),
                error_code: String::new(),
            },
        )
        .map_err(|_| MailBootstrapError::Control)
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), MailBootstrapError> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| MailBootstrapError::Control)
}

fn attachment_blob_admission_publish_permitted(
    permit: &RuntimePublishPermitV1,
) -> Result<bool, MailBootstrapError> {
    let contract = hermes_communications_ingress::admission::communication_attachment_blob_admission_observed_contract_reference_v1();
    let subject = DurableSubjectV1::new(
        StreamKindV1::Observation,
        contract.owner,
        contract.name,
        contract.major,
    )
    .map_err(|_| MailBootstrapError::EventHub)?;
    Ok(permit.permits_subject(&subject))
}

fn bind_attachment_anchor_subscribe_permit(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<Option<RuntimeSubscribePermitV1>, MailBootstrapError> {
    let expected = hermes_communications_ingress::admission::communication_attachment_anchor_recorded_contract_reference_v1();
    let mut anchor = None;
    for permit in permits {
        let Some(contract) = permit.contract() else {
            return Err(MailBootstrapError::EventHub);
        };
        if contract.owner == expected.owner
            && contract.name == expected.name
            && contract.major == expected.major
            && contract.revision == expected.revision
            && contract.schema_sha256 == expected.schema_sha256
        {
            if anchor.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else {
            return Err(MailBootstrapError::EventHub);
        }
    }
    Ok(anchor)
}

fn map_attachment_anchor_mapping_error(
    error: MailAttachmentAnchorMappingErrorV1,
) -> MailBootstrapError {
    let _ = error;
    MailBootstrapError::AttachmentAnchorMapping
}

fn valid_gmail_history_id(value: Option<&str>) -> Option<&str> {
    value.filter(|history_id| {
        !history_id.is_empty() && history_id.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn newer_gmail_history_id<'a>(
    current: Option<&'a str>,
    candidate: Option<&'a str>,
) -> Option<&'a str> {
    match (
        valid_gmail_history_id(current),
        valid_gmail_history_id(candidate),
    ) {
        (None, value) | (value, None) => value,
        (Some(current), Some(candidate))
            if candidate.len() > current.len()
                || (candidate.len() == current.len() && candidate > current) =>
        {
            Some(candidate)
        }
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
        draft_ingress_observation_with_body(
            operation_id,
            provider,
            connection_id,
            source_id,
            BodyAvailabilityV1::Unavailable,
        )
        .map_err(|_| MailBootstrapError::Provider)?,
        failure,
    )
    .map_err(|_| MailBootstrapError::Provider)
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

fn observation_context(
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> ObservationEnvelopeContextV1 {
    ObservationEnvelopeContextV1 {
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        module_id: MAIL_MODULE_ID.to_owned(),
        recorded_at_unix_seconds,
        recorded_at_nanos,
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use hermes_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ModuleClientRequestV1, managed_runtime_control_frame_v2::Frame,
    };
    use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;

    use super::*;

    #[test]
    fn attachment_blob_admission_requires_its_exact_publish_subject() {
        let expected = DurableSubjectV1::new(
            StreamKindV1::Observation,
            "communications",
            "communication_attachment_blob_admission_observed",
            1,
        )
        .expect("subject");
        let permit =
            RuntimePublishPermitV1::new(MAIL_MODULE_ID, "mail-runtime-1", 1, 1, vec![expected])
                .expect("permit");
        assert!(attachment_blob_admission_publish_permitted(&permit).is_ok_and(|value| value));

        let observed_only = RuntimePublishPermitV1::new(
            MAIL_MODULE_ID,
            "mail-runtime-1",
            1,
            1,
            vec![
                DurableSubjectV1::new(
                    StreamKindV1::Observation,
                    "communications",
                    "communication_observed",
                    1,
                )
                .expect("subject"),
            ],
        )
        .expect("permit");
        assert!(
            attachment_blob_admission_publish_permitted(&observed_only).is_ok_and(|value| !value)
        );
    }

    #[test]
    fn observations_use_the_exact_admitted_mail_module_identity() {
        let context = observation_context("mail-runtime-1", 7, 10, 11);

        assert_eq!(context.module_id, MAIL_MODULE_ID);
        assert_eq!(context.runtime_instance_id, "mail-runtime-1");
        assert_eq!(context.runtime_generation, 7);
    }

    #[test]
    fn inbound_identity_is_stable_across_sync_operations_and_distinguishes_parts() {
        let message =
            inbound_observation_id(ProviderProvenanceV1::MailImap, "account-1", "uid-42", None);

        assert_eq!(
            message,
            inbound_observation_id(ProviderProvenanceV1::MailImap, "account-1", "uid-42", None,),
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

    #[test]
    fn nested_client_delivery_gets_a_correlated_busy_response_without_stealing_platform_reply() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        let kernel = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(kernel);
            let (platform_id, _) = channel.receive_request().expect("platform request");
            channel
                .write_request(
                    [7; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::ClientDelivery(
                            ManagedRuntimeClientDeliveryRequestV1 {
                                request: Some(ModuleClientRequestV1 {
                                    protocol_major: 1,
                                    module_id: MAIL_MODULE_ID.to_owned(),
                                    owner_id: MAIL_MODULE_ID.to_owned(),
                                    contract: Some(ContractReferenceV1 {
                                        owner: MAIL_MODULE_ID.to_owned(),
                                        name: "query".to_owned(),
                                        major: 1,
                                        revision: 1,
                                        schema_sha256: vec![1; 32],
                                    }),
                                    request_id: 41,
                                    request_payload: vec![1],
                                }),
                            },
                        )),
                    },
                )
                .expect("client delivery");
            let nested = channel.read_frame().expect("busy response");
            assert_eq!(
                nested.correlation_id,
                vec![7; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            let Some(Frame::Response(response)) = nested.frame else {
                panic!("nested response");
            };
            let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
                panic!("client delivery response");
            };
            assert_eq!(
                delivery.response.expect("module response").error_code,
                "RUNTIME_BUSY"
            );
            channel
                .write_response(
                    platform_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("platform response");
        });

        let mut channel = ManagedControlChannelV2::new(runtime);
        let mut dispatcher = MailBusyControlDispatcher;
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                },
                &mut dispatcher,
            )
            .expect("correlated platform response");
        assert!(matches!(response.result, Some(ControlResult::Ack(_))));
        kernel.join().expect("kernel join");
    }
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn credential_revision(
    admission: &MailRuntimeAdmission,
    purpose: MailCredentialPurpose,
) -> Result<Option<u64>, MailBootstrapError> {
    let revision = match purpose {
        MailCredentialPurpose::ImapPassword => admission.credential_revisions.imap_password,
        MailCredentialPurpose::GmailAccessToken => {
            admission.credential_revisions.gmail_access_token
        }
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
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_provider_credential_error={error:?}");
    }
    match error {
        ManagedProviderCredentialErrorV1::InvalidContext => MailBootstrapError::Admission,
        ManagedProviderCredentialErrorV1::Rejected
        | ManagedProviderCredentialErrorV1::Unavailable => MailBootstrapError::Credential,
    }
}

fn current_unix_seconds() -> Result<i64, MailBootstrapError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MailBootstrapError::Provider)
        .and_then(|elapsed| {
            i64::try_from(elapsed.as_secs()).map_err(|_| MailBootstrapError::Provider)
        })
}
