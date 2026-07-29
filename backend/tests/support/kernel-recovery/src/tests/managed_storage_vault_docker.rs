//! Real managed Vault and Storage binaries over disposable PostgreSQL/PgBouncer.

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use futures_util::StreamExt;
use hermes_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, RuntimeNatsJwtCredentialV1, v1::DurableEnvelopeV1,
};
use hermes_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleEventDeliveryPolicyV1,
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
    ModuleStorageRequestV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    PlatformStorageBundleV1, PlatformStorageEndpointV1, PlatformStorageTopology,
    StorageDeploymentProfileV1,
};
use hermes_runtime_protocol::v1::{
    ManagedDomainRuntimeConfigurationV1, ManagedRuntimeEventCredentialDeliveryV1,
    ManagedRuntimeEventCredentialRequestV1, SchedulerRuntimeControlRequestV1,
    SchedulerRuntimeControlResponseV1, SchedulerScheduleUpsertOutcomeV1, SettingsSchemaRefV1,
    SettingsSchemaV1, UpsertSchedulerScheduleRequestV1,
    scheduler_runtime_control_request_v1::Operation as SchedulerOperation,
    scheduler_runtime_control_response_v1::Result as SchedulerResult,
};
use hermes_scheduler_protocol::v1::ScheduledJobCommandV1;
use hermes_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageRuntimeControlRequestV1,
    StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
    storage_runtime_control_request_v1::Operation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use nats_jwt::KeyPair;
use prost::Message;

use super::common::*;
use crate::identity::device::signer::FileDeviceSigner;
use crate::platform::managed::signed_bundle::{
    InstalledSignedBundle, SignedNativeDependency, SignedRuntimeArtifact,
};
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::platform::vault::owner_derived_key::OwnerDerivedKeyHandlerV1;
use crate::platform::vault::provider_credential::ProviderCredentialHandlerV1;
use crate::platform::vault::status as vault_status;
use crate::platform::vault::{binding as vault_binding, launch as vault_launch};
use crate::platform::{
    blob::{binding as blob_binding, launch as blob_launch, session::BlobSessionHandlerV1},
    events::{catalog as event_catalog, topology as event_topology},
    macos::managed_launch,
    scheduler::{launch as scheduler_launch, lifecycle as scheduler_lifecycle},
    storage::issuance::{StorageBindingIssueV1, issue_managed},
    storage::successor as storage_successor,
};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation,
};

#[path = "managed_storage_vault_docker/shared_fixture.rs"]
mod shared_fixture;
use shared_fixture::*;
#[path = "managed_storage_vault_docker/nats_outage_fixture.rs"]
mod nats_outage_fixture;
use nats_outage_fixture::*;
#[path = "managed_storage_vault_docker/owner_control_fixture.rs"]
mod owner_control_fixture;
use owner_control_fixture::*;
#[path = "managed_storage_vault_docker/scheduler_setup.rs"]
mod scheduler_setup;
use scheduler_setup::*;
#[path = "managed_storage_vault_docker/scheduler_events.rs"]
mod scheduler_events;
use scheduler_events::*;
#[path = "managed_storage_vault_docker/communications_setup.rs"]
mod communications_setup;
use communications_setup::*;
#[path = "managed_storage_vault_docker/communications_export_race.rs"]
mod communications_export_race;
use communications_export_race::*;
#[path = "managed_storage_vault_docker/communications_backup.rs"]
mod communications_backup;
use communications_backup::*;
#[path = "managed_storage_vault_docker/telegram_event_flow.rs"]
mod telegram_event_flow;
use telegram_event_flow::*;
#[path = "managed_storage_vault_docker/telegram_managed_setup.rs"]
mod telegram_managed_setup;
use telegram_managed_setup::*;
#[path = "managed_storage_vault_docker/attachment_security_blob_fixture.rs"]
mod attachment_security_blob_fixture;
#[path = "managed_storage_vault_docker/attachment_security_clamav_fixture.rs"]
mod attachment_security_clamav_fixture;
#[path = "managed_storage_vault_docker/attachment_security_event_flow.rs"]
mod attachment_security_event_flow;
#[path = "managed_storage_vault_docker/attachment_security_managed_flow.rs"]
mod attachment_security_managed_flow;
#[path = "managed_storage_vault_docker/attachment_security_managed_setup.rs"]
mod attachment_security_managed_setup;
#[path = "managed_storage_vault_docker/attachment_security_persistence_fixture.rs"]
mod attachment_security_persistence_fixture;
#[path = "managed_storage_vault_docker/mail_attachment_flow.rs"]
mod mail_attachment_flow;
#[path = "managed_storage_vault_docker/mail_composition_flow.rs"]
mod mail_composition_flow;
#[path = "managed_storage_vault_docker/mail_delivery_test_support.rs"]
mod mail_delivery_test_support;
#[path = "managed_storage_vault_docker/mail_event_flow.rs"]
mod mail_event_flow;
#[path = "managed_storage_vault_docker/mail_gmail_fixture.rs"]
mod mail_gmail_fixture;
#[path = "managed_storage_vault_docker/mail_gmail_oauth_fixture.rs"]
mod mail_gmail_oauth_fixture;
#[path = "managed_storage_vault_docker/mail_imap_fixture.rs"]
mod mail_imap_fixture;
#[path = "managed_storage_vault_docker/mail_managed_setup.rs"]
mod mail_managed_setup;
#[path = "managed_storage_vault_docker/mail_operational_flow.rs"]
mod mail_operational_flow;
#[path = "managed_storage_vault_docker/mail_smtp_fixture.rs"]
mod mail_smtp_fixture;
#[path = "managed_storage_vault_docker/mail_sync_health_flow.rs"]
mod mail_sync_health_flow;
use mail_attachment_flow::*;
use mail_delivery_test_support::*;
use mail_event_flow::*;
use mail_gmail_fixture::*;
use mail_gmail_oauth_fixture::*;
use mail_imap_fixture::*;
use mail_smtp_fixture::*;
#[path = "managed_storage_vault_docker/telegram_managed_flow.rs"]
mod telegram_managed_flow;
use attachment_security_blob_fixture::*;
use attachment_security_event_flow::*;
use attachment_security_managed_setup::*;
use mail_managed_setup::*;
use mail_operational_flow::*;
use mail_sync_health_flow::*;
#[path = "managed_storage_vault_docker/mail_account_credential_flow.rs"]
mod mail_account_credential_flow;
#[path = "managed_storage_vault_docker/mail_delivery_flow.rs"]
mod mail_delivery_flow;
#[path = "managed_storage_vault_docker/mail_gmail_delivery_flow.rs"]
mod mail_gmail_delivery_flow;
#[path = "managed_storage_vault_docker/mail_gmail_oauth_flow.rs"]
mod mail_gmail_oauth_flow;
#[path = "managed_storage_vault_docker/mail_managed_flow.rs"]
mod mail_managed_flow;
#[path = "managed_storage_vault_docker/mail_message_flag_flow.rs"]
mod mail_message_flag_flow;
#[path = "managed_storage_vault_docker/mail_message_location_flow.rs"]
mod mail_message_location_flow;
#[path = "managed_storage_vault_docker/mail_message_permanent_delete_flow.rs"]
mod mail_message_permanent_delete_flow;
#[path = "managed_storage_vault_docker/mail_outbound_attachment_flow.rs"]
mod mail_outbound_attachment_flow;
#[path = "managed_storage_vault_docker/zulip_https_fixture.rs"]
mod zulip_https_fixture;
use zulip_https_fixture::*;
#[path = "managed_storage_vault_docker/zulip_managed_setup.rs"]
mod zulip_managed_setup;
use zulip_managed_setup::*;
#[path = "managed_storage_vault_docker/zulip_managed_fixture.rs"]
mod zulip_managed_fixture;
use zulip_managed_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_managed_setup.rs"]
mod whatsapp_managed_setup;
#[path = "managed_storage_vault_docker/zulip_event_flow.rs"]
mod zulip_event_flow;
#[path = "managed_storage_vault_docker/zulip_managed_flow.rs"]
mod zulip_managed_flow;
use whatsapp_managed_setup::*;
#[path = "managed_storage_vault_docker/whatsapp_managed_fixture.rs"]
mod whatsapp_managed_fixture;
use whatsapp_managed_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_host_fixture.rs"]
mod whatsapp_host_fixture;
use whatsapp_host_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_event_flow.rs"]
mod whatsapp_event_flow;
#[path = "managed_storage_vault_docker/whatsapp_managed_flow.rs"]
mod whatsapp_managed_flow;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault and Storage binaries"]
fn managed_storage_binary_bootstraps_through_live_vault() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-storage-vault-docker");
    let data = private_directory(root.join("kernel"));
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let release = installed_release(&root);
    let store = Arc::new(configured_store(&root, release.kernel()));
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    assert_eq!(
        start_vault(&supervisor, &store, &data, release.kernel()),
        1,
        "Vault starts from the signed release binding"
    );
    let vault =
        vault_status::read_current(&store, &supervisor.relay_port()).expect("live Vault status");
    assert_eq!(vault.runtime_generation(), 1);
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        1,
        "Storage starts from the signed release binding"
    );
    assert_reconciling_status(&supervisor, 1);
    supervisor.stop("storage").expect("stop Storage");
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        2,
        "restarted Storage re-verifies the signed release binding"
    );
    assert_reconciling_status(&supervisor, 2);
    supervisor.shutdown().expect("stop managed processes");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Scheduler and NATS binaries"]
fn managed_scheduler_crash_uses_storage_control_successor_provisioning() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let fixture = SchedulerRecoveryFixture::start();
    let binding = fixture.start_initial_scheduler();
    let due_at = fixture.persist_recovery_schedule();
    let worker = fixture.restart_after_crash(due_at);
    let successor = fixture.assert_successor(&binding, due_at);
    fixture.assert_revoked_binding_does_not_restart(successor);
    fixture.shutdown(worker);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Communications binaries"]
fn managed_communications_domain_starts_with_owner_local_storage_and_events() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-communications-domain");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Communications Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime")
        )
        .expect("start signed Blob runtime"),
        1,
        "Blob starts as a separate managed platform process"
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    configure_communications_jetstream(&store);

    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1,
        "generic managed-domain launch admits Communications without a Kernel owner facade"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("read Communications process state")
    );
    assert_communications_ingress_delivery(&store, &supervisor);
    assert_communications_relationship_projection(&store, &supervisor);
    assert_communications_attachment_anchor_projection(&store, &supervisor);
    let _ = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        true,
    );
    assert_communications_query_delivery(&store, &supervisor);
    assert_communications_canonical_read_v2_pagination(&store, &supervisor);
    assert_communications_search_query_delivery(&store, &supervisor);
    assert_communications_gateway_query_delivery(&store, &supervisor, &root, &data);
    assert_telegram_outbox_delivery(&store, &supervisor);
    assert_fenced_communications_target_cannot_issue_blob_custody_grant(&store, &supervisor, &data);

    supervisor.shutdown().expect("stop managed processes");
    assert_communications_storage_backup_restore(&root);
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Blob and Communications Export workflow binaries"]
fn managed_communications_export_workflow_starts_with_owner_local_storage_and_events() {
    use hermes_communications_evidence_export_source_api::wire::EvidenceExportRejectCodeV1;
    use hermes_communications_export_api::{
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1, COMMUNICATIONS_EXPORT_MODULE_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        wire::{
            CommunicationsExportErrorCodeV1, EvidenceExportArtifactReadRequestV1,
            EvidenceExportStatusV1, GetEvidenceExportStatusRequestV1,
            GetEvidenceExportStatusResponseV1, IssueEvidenceExportReadRequestV1,
            IssueEvidenceExportReadResponseV1, StartEvidenceExportRequestV1,
            StartEvidenceExportResponseV1,
        },
    };
    use hermes_communications_export_runtime::admission::{
        communications_export_command_contract_reference_v1,
        communications_export_query_contract_reference_v1,
        communications_export_ticket_contract_reference_v1,
    };
    use hermes_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-communications-export");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let revision_race = Arc::new(CommunicationsExportRevisionRaceV1::new());
    let race_blob_session_handler = Arc::new(CommunicationsExportRaceBlobSessionHandlerV1::new(
        Arc::clone(&store),
        supervisor.relay_port(),
        data.clone(),
        Arc::clone(&revision_race),
    ));
    configure_route_handlers(&supervisor, &store, &data, race_blob_session_handler);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Export Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start signed Blob runtime");
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    configure_communications_jetstream(&store);
    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1,
        "Communications source owner starts independently before its export workflow"
    );
    let message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    issue_initial_communications_export_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_export_storage_binding(&store),
    )
    .expect("provision Communications Export Storage binding after the source-owner recovery");
    assert_eq!(
        start_communications_export_workflow(&supervisor, &store, &root.join("runtime")),
        1,
        "generic managed-workflow launch admits Communications Export without a Kernel owner facade"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_EXPORT_REGISTRATION)
            .expect("read Communications Export process state")
    );
    let route_as = |request_id: u64,
                    logical_owner_id: &str,
                    contract: hermes_runtime_protocol::v1::ContractReferenceV1,
                    request_payload: Vec<u8>| {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: COMMUNICATIONS_EXPORT_MODULE_ID_V1.to_owned(),
            owner_id: COMMUNICATIONS_EXPORT_OWNER_V1.to_owned(),
            contract: Some(contract),
            request_id,
            request_payload,
            logical_owner_id: logical_owner_id.to_owned(),
        }
        .encode_to_vec();
        let launch = store
            .effective_managed_launch_record(COMMUNICATIONS_EXPORT_REGISTRATION)
            .expect("read Communications Export launch")
            .expect("Communications Export launch is active");
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            launch.runtime_instance_id(),
            launch.runtime_generation(),
            launch.grant_epoch(),
            COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
            &request,
        );
        let bytes = crate::modules::capability::router::route_managed_client_request(
            store.as_ref(),
            &supervisor.relay_port(),
            &route,
        )
        .expect("route exact Communications Export client request");
        let response = ModuleClientResponseV1::decode(bytes.as_slice())
            .expect("decode Communications Export module response");
        assert_eq!(response.request_id, request_id);
        assert!(
            response.error_code.is_empty(),
            "Communications Export request {request_id} failed: {}",
            response.error_code,
        );
        response.response_payload
    };
    let route = |request_id: u64,
                 contract: hermes_runtime_protocol::v1::ContractReferenceV1,
                 request_payload: Vec<u8>| {
        route_as(request_id, "owner-1", contract, request_payload)
    };
    let export_id = [11; 16];
    let start = StartEvidenceExportResponseV1::decode(
        route(
            1,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted Communications Export command");
    assert_eq!(start.export_id, export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                2,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusReady as i32 {
            assert_eq!(status.requested_items, 1);
            assert_eq!(status.completed_items, 1);
            assert!(status.artifact_bytes > 0);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "durable Communications Export command did not reach a ready artifact; status={status:?}; runtime_failure={:?}",
            supervisor
                .last_failure(COMMUNICATIONS_EXPORT_REGISTRATION)
                .expect("read Communications Export runtime failure"),
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let wrong_owner_status = GetEvidenceExportStatusResponseV1::decode(
        route_as(
            20,
            "owner-2",
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode wrong-owner Communications Export status");
    assert_eq!(
        wrong_owner_status.status,
        EvidenceExportStatusV1::EvidenceExportStatusUnspecified as i32
    );
    assert_eq!(wrong_owner_status.artifact_bytes, 0);
    assert_eq!(
        wrong_owner_status.error,
        CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound as i32
    );
    let wrong_owner_ticket = IssueEvidenceExportReadResponseV1::decode(
        route_as(
            21,
            "owner-2",
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode wrong-owner Communications Export ticket response");
    assert!(wrong_owner_ticket.opaque_read_capability.is_empty());
    assert_eq!(
        wrong_owner_ticket.error,
        CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound as i32
    );
    let edited_body = publish_and_wait_for_communications_message_edit(
        &store,
        &supervisor,
        &data,
        &message_id,
        b"fixture edited source body for custody transfer".to_vec(),
        1_783_024_009,
        10,
    );
    let edited_export_id = [15; 16];
    let edited_start = StartEvidenceExportResponseV1::decode(
        route(
            12,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: edited_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted edited-message export command");
    assert_eq!(edited_start.export_id, edited_export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                13,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: edited_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode edited-message Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusReady as i32 {
            assert_eq!(status.requested_items, 1);
            assert_eq!(status.completed_items, 1);
            assert!(status.artifact_bytes > 0);
            break;
        }
        assert!(
            status.status != EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
                && std::time::Instant::now() < deadline,
            "edited canonical snapshot must reach ready export status; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let stale_runtime_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            15,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue edited export ticket before workflow restart");
    assert_eq!(stale_runtime_ticket.opaque_read_capability.len(), 32);
    assert_eq!(
        restart_communications_export_workflow(&supervisor, &store, &root.join("runtime")),
        2,
        "Communications Export restart advances its independent runtime generation"
    );
    let restarted_status = GetEvidenceExportStatusResponseV1::decode(
        route(
            16,
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export status after successor restart");
    assert_eq!(
        restarted_status.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(restarted_status.requested_items, 1);
    assert_eq!(restarted_status.completed_items, 1);
    assert!(restarted_status.artifact_bytes > 0);
    set_authenticated_nats_container_running(false);
    let outage_export_id = [16; 16];
    let outage_start = StartEvidenceExportResponseV1::decode(
        route(
            17,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: outage_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export command accepted during NATS outage");
    assert_eq!(outage_start.export_id, outage_export_id);
    std::thread::sleep(std::time::Duration::from_millis(1_500));
    let outage_pending = GetEvidenceExportStatusResponseV1::decode(
        route(
            18,
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: outage_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export status during NATS outage");
    let export_runtime_active_during_outage = supervisor
        .is_active(COMMUNICATIONS_EXPORT_REGISTRATION)
        .expect("read Communications Export process state during NATS outage");
    set_authenticated_nats_container_running(true);
    assert_eq!(
        outage_pending.status,
        EvidenceExportStatusV1::EvidenceExportStatusPendingSource as i32,
        "NATS outage retains the exact export request before source preparation"
    );
    assert!(
        export_runtime_active_during_outage,
        "NATS outage is retryable and does not stop Communications Export"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                19,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: outage_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode Communications Export status after NATS recovery");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusReady as i32 {
            assert_eq!(status.requested_items, 1);
            assert_eq!(status.completed_items, 1);
            assert!(status.artifact_bytes > 0);
            break;
        }
        assert!(
            status.status != EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
                && std::time::Instant::now() < deadline,
            "persisted export request must resume after NATS recovery; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let stale_revision_export_id = [18; 16];
    let communications_database_id = crate::platform::storage::topology::current(&store)
        .expect("read Communications Storage topology")
        .database_id()
        .to_owned();
    revision_race.arm(&communications_database_id, &message_id);
    let stale_revision_start = StartEvidenceExportResponseV1::decode(
        route(
            24,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: stale_revision_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted stale-revision export command");
    assert_eq!(stale_revision_start.export_id, stale_revision_export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                25,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: stale_revision_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode stale-revision Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusRejected as i32 {
            assert_eq!(status.completed_items, 0);
            assert_eq!(status.artifact_bytes, 0);
            assert_eq!(
                status.error,
                CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodePolicyRejected as i32,
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "canonical revision race must reach terminal rejected export status; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        revision_race.fired_revision() > 1,
        "managed source preparation must cross the injected canonical revision fence"
    );
    assert_eq!(
        communications_export_rejection_code(
            &communications_database_id,
            &stale_revision_export_id,
        ),
        EvidenceExportRejectCodeV1::EvidenceExportRejectCodeStaleRevision as u16,
        "workflow terminal state must preserve the typed STALE_REVISION source result",
    );
    let invalid_utf8_body = vec![0xf0, 0x28, 0x8c, 0x28];
    assert_eq!(
        publish_and_wait_for_communications_message_edit(
            &store,
            &supervisor,
            &data,
            &message_id,
            invalid_utf8_body.clone(),
            1_783_024_010,
            11,
        ),
        invalid_utf8_body,
    );
    let invalid_utf8_export_id = [17; 16];
    let invalid_utf8_start = StartEvidenceExportResponseV1::decode(
        route(
            22,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: invalid_utf8_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted invalid-UTF8 export command");
    assert_eq!(invalid_utf8_start.export_id, invalid_utf8_export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                23,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: invalid_utf8_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode invalid-UTF8 Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusRejected as i32 {
            assert_eq!(status.completed_items, 0);
            assert_eq!(status.artifact_bytes, 0);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "invalid UTF-8 canonical body must reach terminal rejected export status; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    publish_and_wait_for_communications_message_deletion(store.as_ref(), &supervisor, &message_id);
    let deleted_export_id = [13; 16];
    let deleted_start = StartEvidenceExportResponseV1::decode(
        route(
            8,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: deleted_export_id.to_vec(),
                message_ids: vec![message_id],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted deleted-message export command");
    assert_eq!(deleted_start.export_id, deleted_export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                9,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: deleted_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode deleted-message Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusRejected as i32 {
            assert_eq!(status.completed_items, 0);
            assert_eq!(status.artifact_bytes, 0);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "deleted canonical message must reach terminal rejected export status; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let rejected_export_id = [12; 16];
    let rejected_start = StartEvidenceExportResponseV1::decode(
        route(
            4,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: rejected_export_id.to_vec(),
                message_ids: vec![vec![99; 16]],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted unknown-message export command");
    assert_eq!(rejected_start.export_id, rejected_export_id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let status = GetEvidenceExportStatusResponseV1::decode(
            route(
                5,
                communications_export_query_contract_reference_v1(),
                GetEvidenceExportStatusRequestV1 {
                    protocol_major: 1,
                    export_id: rejected_export_id.to_vec(),
                }
                .encode_to_vec(),
            )
            .as_slice(),
        )
        .expect("decode rejected Communications Export status");
        if status.status == EvidenceExportStatusV1::EvidenceExportStatusRejected as i32 {
            assert_eq!(status.completed_items, 0);
            assert_eq!(status.artifact_bytes, 0);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "unknown or deleted canonical message must reach terminal rejected export status; status={status:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            3,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode one-use Communications Export read ticket");
    assert_eq!(ticket.opaque_read_capability.len(), 32);
    assert!(ticket.declared_bytes > 0);
    let edited_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            14,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode edited-message Communications Export read ticket");
    assert_eq!(edited_ticket.opaque_read_capability.len(), 32);
    assert!(edited_ticket.declared_bytes > 0);
    let blob_outage_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            6,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue read ticket before Blob outage");
    let browser_cookie = assert_communications_export_gateway_delivery(
        &store,
        &supervisor,
        &root,
        &data,
        release.kernel(),
        CommunicationsExportGatewayDeliveryInputsV1 {
            opaque_read_capability: ticket.opaque_read_capability,
            declared_bytes: ticket.declared_bytes,
            edited_body: &edited_body,
            edited_opaque_read_capability: edited_ticket.opaque_read_capability,
            edited_declared_bytes: edited_ticket.declared_bytes,
            stale_runtime_read_capability: stale_runtime_ticket.opaque_read_capability,
            blob_outage_read_capability: blob_outage_ticket.opaque_read_capability,
        },
    );
    let revoked_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            7,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue read ticket before export workflow revoke");
    store
        .transition_module_registration(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            ModuleRegistrationState::Revoked,
        )
        .expect("revoke Communications Export workflow registration");
    assert_communications_export_gateway_rejects_revoked_ticket(
        &store,
        &supervisor,
        &root,
        &data,
        revoked_ticket.opaque_read_capability,
        &browser_cookie,
    );
    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

struct CommunicationsExportGatewayDeliveryInputsV1<'a> {
    opaque_read_capability: Vec<u8>,
    declared_bytes: u64,
    edited_body: &'a [u8],
    edited_opaque_read_capability: Vec<u8>,
    edited_declared_bytes: u64,
    stale_runtime_read_capability: Vec<u8>,
    blob_outage_read_capability: Vec<u8>,
}

fn assert_communications_export_gateway_delivery(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &std::path::Path,
    kernel_data: &std::path::Path,
    kernel_executable: &std::path::Path,
    inputs: CommunicationsExportGatewayDeliveryInputsV1<'_>,
) -> String {
    use hermes_communications_export_api::{
        COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, wire::EvidenceExportArtifactReadRequestV1,
    };
    use http_body_util::BodyExt as _;

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(store),
        kernel_data,
        supervisor.clone(),
        &configuration,
        None,
    )
    .expect("compose owner Gateway routes");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway test runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let stale_runtime_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.stale_runtime_read_capability,
    }
    .encode_to_vec();
    let stale_runtime_response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                .header("content-type", "application/proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    stale_runtime_read_request,
                )))
                .expect("Gateway stale-runtime Communications Export artifact read request"),
        ),
    );
    assert_eq!(
        stale_runtime_response.status(),
        hyper::StatusCode::NOT_FOUND,
        "workflow restart invalidates predecessor runtime-local read tickets"
    );
    let read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.opaque_read_capability,
    }
    .encode_to_vec();
    let read = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", &cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        read_request.clone(),
                    )))
                    .expect("Gateway Communications Export artifact read request"),
            ),
        )
    };
    let response = read();
    assert_eq!(response.status(), hyper::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(response.headers().get("x-blob-reference").is_none());
    assert!(response.headers().get("digest").is_none());
    let artifact = runtime
        .block_on(response.into_body().collect())
        .expect("Gateway Communications Export artifact response")
        .to_bytes();
    assert_eq!(
        u64::try_from(artifact.len()).ok(),
        Some(inputs.declared_bytes)
    );
    assert!(artifact.starts_with(
        br#"{"record_type":"manifest","schema":"hermes.communications.evidence-export.v1"#
    ));
    assert!(
        artifact
            .windows(br#""logical_owner_id":"owner-1""#.len())
            .any(|window| window == br#""logical_owner_id":"owner-1""#),
        "artifact manifest carries the exact logical owner provenance"
    );
    assert!(
        artifact
            .windows(b"fixture source body for custody transfer".len())
            .any(|window| window == b"fixture source body for custody transfer")
    );
    assert!(
        !artifact
            .windows(inputs.edited_body.len())
            .any(|window| window == inputs.edited_body),
        "pre-edit export artifact remains bound to its original canonical snapshot"
    );
    assert_eq!(read().status(), hyper::StatusCode::NOT_FOUND);
    let edited_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.edited_opaque_read_capability,
    }
    .encode_to_vec();
    let read_edited = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", &cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        edited_read_request.clone(),
                    )))
                    .expect("Gateway edited Communications Export artifact read request"),
            ),
        )
    };
    let edited_response = read_edited();
    assert_eq!(edited_response.status(), hyper::StatusCode::OK);
    let edited_artifact = runtime
        .block_on(edited_response.into_body().collect())
        .expect("Gateway edited Communications Export artifact response")
        .to_bytes();
    assert_eq!(
        u64::try_from(edited_artifact.len()).ok(),
        Some(inputs.edited_declared_bytes)
    );
    assert!(
        edited_artifact
            .windows(inputs.edited_body.len())
            .any(|window| window == inputs.edited_body),
        "post-edit export artifact contains the edited canonical snapshot"
    );
    assert_eq!(read_edited().status(), hyper::StatusCode::NOT_FOUND);
    let blob_outage_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.blob_outage_read_capability,
    }
    .encode_to_vec();
    let read_during_blob_outage = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", &cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        blob_outage_read_request.clone(),
                    )))
                    .expect("Gateway Communications Export Blob-outage read request"),
            ),
        )
    };

    supervisor
        .stop("blob")
        .expect("stop Blob for Communications Export artifact outage");
    assert_eq!(
        read_during_blob_outage().status(),
        hyper::StatusCode::SERVICE_UNAVAILABLE,
        "Blob outage fails closed without disclosing Communications Export artifact bytes"
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            supervisor,
            store,
            kernel_executable,
            kernel_data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Communications Export artifact outage"),
        2
    );
    assert_eq!(
        read_during_blob_outage().status(),
        hyper::StatusCode::NOT_FOUND,
        "artifact ticket is consumed atomically before the failed Blob read and cannot be replayed"
    );
    cookie
}

fn assert_communications_export_gateway_rejects_revoked_ticket(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &std::path::Path,
    kernel_data: &std::path::Path,
    opaque_read_capability: Vec<u8>,
    cookie: &str,
) {
    use hermes_communications_export_api::{
        COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, wire::EvidenceExportArtifactReadRequestV1,
    };

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(store),
        kernel_data,
        supervisor.clone(),
        &configuration,
        None,
    )
    .expect("compose Gateway after export workflow revoke");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway test runtime");
    let response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                .header("content-type", "application/proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    EvidenceExportArtifactReadRequestV1 {
                        opaque_read_capability,
                    }
                    .encode_to_vec(),
                )))
                .expect("Gateway revoked Communications Export artifact read request"),
        ),
    );
    assert_eq!(
        response.status(),
        hyper::StatusCode::NOT_FOUND,
        "revoke removes the exact export client_blob route before any artifact read"
    );
}

fn short_communications_kernel_data_directory() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    PathBuf::from("/tmp").join(format!("hermes-comms-{}-{suffix}", std::process::id()))
}

fn assert_communications_gateway_query_delivery(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &std::path::Path,
    kernel_data: &std::path::Path,
) {
    use http_body_util::BodyExt as _;

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(store),
        kernel_data,
        supervisor.clone(),
        &configuration,
        None,
    )
    .expect("compose owner Gateway routes");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway test runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let route_query =
        |request: hermes_communications_api::query_wire::CommunicationsQueryRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri("/hermes.communications.query.v1.CommunicationsQueryService/Query")
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway owner query request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            assert_eq!(
                response
                    .headers()
                    .get("content-type")
                    .and_then(|value| value.to_str().ok()),
                Some("application/proto"),
            );
            assert_eq!(
                response
                    .headers()
                    .get("connect-protocol-version")
                    .and_then(|value| value.to_str().ok()),
                Some("1"),
            );
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway owner query response")
                .to_bytes();
            hermes_communications_api::query_wire::CommunicationsQueryResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications query response")
        };
    let route_saved_search =
        |request: hermes_communications_saved_query_api::CommunicationsSavedSearchRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri(hermes_communications_saved_query_api::SAVED_SEARCH_CONNECT_PATH_V1)
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway Communications saved-search request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway Communications saved-search response")
                .to_bytes();
            hermes_communications_saved_query_api::CommunicationsSavedSearchResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications saved-search response")
        };
    let route_sender_insights =
        |request: hermes_communications_sender_insights_api::ListSenderInsightsRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri(
                            hermes_communications_sender_insights_api::SENDER_INSIGHTS_CONNECT_PATH_V1,
                        )
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway Communications sender-insights request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway Communications sender-insights response")
                .to_bytes();
            hermes_communications_sender_insights_api::ListSenderInsightsResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications sender-insights response")
        };
    let response = route_query(
        hermes_communications_api::query_wire::CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(
                hermes_communications_api::query_wire::communications_query_request_v1::Operation::ListAccounts(
                    hermes_communications_api::query_wire::ListAccountsRequestV1 {
                        limit: 16,
                        cursor: Vec::new(),
                    },
                ),
            ),
        },
    );
    assert!(matches!(
        response.result,
        Some(hermes_communications_api::query_wire::communications_query_response_v1::Result::ListAccounts(accounts))
            if !accounts.accounts.is_empty()
    ));

    let response = route_query(
        hermes_communications_api::query_wire::CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(
                hermes_communications_api::query_wire::communications_query_request_v1::Operation::SearchCommunications(
                    hermes_communications_api::query_wire::SearchCommunicationsRequestV1 {
                        query: "fixture".to_owned(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                ),
            ),
        },
    );
    assert!(response.error_code.is_empty());
    assert!(matches!(
        &response.result,
        Some(hermes_communications_api::query_wire::communications_query_response_v1::Result::SearchCommunications(hits))
            if !hits.hits.is_empty()
                && hits.hits.iter().all(|hit| {
                    hit.evidence_id.len() == 16
                        && hit.message_id.len() == 16
                        && hit.conversation_id.len() == 16
                        && hit.matched_token_count > 0
                })
    ));
    let message_id = match &response.result {
        Some(
            hermes_communications_api::query_wire::communications_query_response_v1::Result::SearchCommunications(
                hits,
            ),
        ) => hits
            .hits
            .iter()
            .find_map(|hit| {
                let detail = route_query(
                    hermes_communications_api::query_wire::CommunicationsQueryRequestV1 {
                        protocol_major: 1,
                        operation: Some(
                            hermes_communications_api::query_wire::communications_query_request_v1::Operation::GetMessage(
                                hermes_communications_api::query_wire::GetMessageRequestV1 {
                                    message_id: hit.message_id.clone(),
                                },
                            ),
                        ),
                    },
                );
                matches!(
                    detail.result,
                    Some(
                        hermes_communications_api::query_wire::communications_query_response_v1::Result::GetMessage(
                            hermes_communications_api::query_wire::GetMessageResponseV1 {
                                message: Some(ref message),
                            },
                        ),
                    ) if message.body_state == 4
                )
                .then(|| hit.message_id.clone())
            })
            .expect("search result includes the admitted canonical body"),
        _ => unreachable!("search result checked above"),
    };
    let public_payload = response.encode_to_vec();
    for private_value in [
        "fixture source body for custody transfer",
        "blob://fixture-source/admitted-body-1",
    ] {
        assert!(
            !public_payload
                .windows(private_value.len())
                .any(|window| window == private_value.as_bytes()),
            "external Communications search must not reveal private body or Blob locator",
        );
    }

    let sender_insights = route_sender_insights(
        hermes_communications_sender_insights_api::ListSenderInsightsRequestV1 {
            protocol_major: 1,
            account_id: None,
            limit: 20,
            cursor: Vec::new(),
        },
    );
    assert_eq!(
        sender_insights.error,
        hermes_communications_sender_insights_api::SenderInsightsErrorCodeV1::SenderInsightsErrorCodeUnspecified
            as i32
    );
    let sender_insight = sender_insights
        .items
        .iter()
        .find(|item| item.display_label.as_deref() == Some("Fixture Sender <sender@example.test>"))
        .expect("managed sender projection contains the admitted Mail fixture sender");
    assert_eq!(sender_insight.sender_id.len(), 16);
    assert_eq!(
        sender_insight.display_label.as_deref(),
        Some("Fixture Sender <sender@example.test>")
    );
    assert_eq!(sender_insight.message_count, 1);
    assert_eq!(sender_insight.conversation_count, 1);
    assert!(sender_insight.first_observed_at_unix_seconds > 0);
    assert!(
        sender_insight.last_observed_at_unix_seconds
            >= sender_insight.first_observed_at_unix_seconds
    );
    let sender_insights_payload = sender_insights.encode_to_vec();
    for private_value in [
        "integration-private-account-1",
        "integration-private-record-1",
        "fixture source body for custody transfer",
        "blob://fixture-source/admitted-body-1",
    ] {
        assert!(
            !sender_insights_payload
                .windows(private_value.len())
                .any(|window| window == private_value.as_bytes()),
            "sender-insights response must not reveal provider locators or message content",
        );
    }

    use hermes_communications_saved_query_api::{
        CommunicationsSavedSearchRequestV1, CreateSavedSearchRequestV1, DeleteSavedSearchRequestV1,
        ExecuteSavedSearchRequestV1, ListSavedSearchesRequestV1, ReplaceSavedSearchRequestV1,
        SavedSearchErrorCodeV1,
        communications_saved_search_request_v1::Operation as SavedSearchOperation,
        communications_saved_search_response_v1::Result as SavedSearchResult,
    };
    let saved_search_id = vec![0x31; 16];
    let create = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Create(CreateSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            name: "Fixture evidence".to_owned(),
            description: Some("Managed conformance definition".to_owned()),
            account_id: None,
            query: "fixture".to_owned(),
        })),
    });
    assert_eq!(
        create.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeUnspecified as i32
    );
    assert!(matches!(
        create.result,
        Some(SavedSearchResult::Mutation(ref mutation))
            if matches!(mutation.item, Some(ref item) if item.revision == 1)
    ));

    let list = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::List(ListSavedSearchesRequestV1 {
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert!(matches!(
        list.result,
        Some(SavedSearchResult::List(ref page))
            if page.items.iter().any(|item| item.saved_search_id == saved_search_id)
    ));

    let execute = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Execute(ExecuteSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert!(matches!(
        execute.result,
        Some(SavedSearchResult::Execute(ref page))
            if page.definition_revision == 1 && !page.hits.is_empty()
    ));
    let saved_search_payload = execute.encode_to_vec();
    assert!(
        !saved_search_payload
            .windows("fixture".len())
            .any(|window| window == b"fixture"),
        "saved-search responses must not reveal query plaintext"
    );

    let stale_replace = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Replace(ReplaceSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            expected_revision: 99,
            name: "Fixture evidence".to_owned(),
            description: None,
            account_id: None,
            query: "fixture".to_owned(),
        })),
    });
    assert_eq!(
        stale_replace.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeRevisionConflict as i32
    );

    let deleted = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Delete(DeleteSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            expected_revision: 1,
        })),
    });
    assert!(matches!(
        deleted.result,
        Some(SavedSearchResult::Delete(ref result))
            if result.saved_search_id == saved_search_id && result.revision == 2
    ));
    let missing = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Execute(ExecuteSavedSearchRequestV1 {
            saved_search_id,
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert_eq!(
        missing.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeNotFound as i32
    );

    let ticket_response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(hermes_communications_content_api::CONTENT_TICKET_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    hermes_communications_content_api::IssueMessageBodyReadRequestV1 {
                        protocol_major: 1,
                        message_id,
                    }
                    .encode_to_vec(),
                )))
                .expect("Gateway Communications content ticket request"),
        ),
    );
    assert_eq!(ticket_response.status(), hyper::StatusCode::OK);
    let ticket_bytes = runtime
        .block_on(ticket_response.into_body().collect())
        .expect("Gateway Communications content ticket response")
        .to_bytes();
    let ticket = hermes_communications_content_api::IssueMessageBodyReadResponseV1::decode(
        ticket_bytes.as_ref(),
    )
    .expect("decode Communications content ticket");
    assert!(ticket.error_code.is_empty());
    assert_eq!(ticket.opaque_read_capability.len(), 32);
    assert_eq!(
        ticket.declared_bytes,
        u64::try_from("fixture source body for custody transfer".len()).expect("fixture body size")
    );
    let read_request = hermes_communications_content_api::ReadMessageBodyRequestV1 {
        protocol_major: 1,
        opaque_read_capability: ticket.opaque_read_capability,
    }
    .encode_to_vec();
    let read = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(hermes_communications_content_api::CONTENT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", &cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        read_request.clone(),
                    )))
                    .expect("Gateway Communications content read request"),
            ),
        )
    };
    let content = read();
    assert_eq!(content.status(), hyper::StatusCode::OK);
    assert_eq!(
        content
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(content.headers().get("x-blob-reference").is_none());
    assert!(content.headers().get("digest").is_none());
    assert_eq!(
        runtime
            .block_on(content.into_body().collect())
            .expect("Gateway Communications content response")
            .to_bytes()
            .as_ref(),
        b"fixture source body for custody transfer"
    );
    assert_eq!(read().status(), hyper::StatusCode::NOT_FOUND);
}

struct SchedulerRecoveryFixture {
    root: PathBuf,
    release: InstalledSignedBundle,
    store: Arc<SqliteControlStore>,
    shutdown: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
}

impl SchedulerRecoveryFixture {
    fn start() -> Self {
        let root = unique_target_root("hermes-managed-scheduler-lifecycle");
        let data = private_directory(root.join("kernel"));
        initialize_vault(
            &private_directory(data.join("vault")),
            &credential_directory(),
        );
        let release = installed_scheduler_release(&root);
        let store = Arc::new(configured_scheduler_store(&root, release.kernel()));
        let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
        let shutdown = Arc::new(AtomicBool::new(false));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
        configure_route_handler(&supervisor, &store, &data);
        supervisor
            .configure_event_credential_handler(Arc::new(
                UnauthenticatedNatsCredentialHandler::new(Arc::clone(&store)),
            ))
            .expect("configure Scheduler Event credential handler");
        start_vault(&supervisor, &store, &data, release.kernel());
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        );
        issue_initial_scheduler_storage_binding(&store);
        crate::platform::storage::provisioning::apply_reserved_binding(
            &supervisor,
            &store,
            &scheduler_binding(&store),
        )
        .unwrap_or_else(|error| panic!("provision initial Scheduler Storage binding: {error:?}"));
        configure_scheduler_jetstream(&store);
        configure_scheduler_delivery_observer(&store);
        Self {
            root,
            release,
            store,
            shutdown,
            supervisor,
        }
    }

    fn start_initial_scheduler(&self) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
        let reservation =
            managed_launch::load(&self.supervisor, &self.store, SCHEDULER_REGISTRATION)
                .expect("load initial Scheduler reservation");
        let binding = scheduler_binding(&self.store);
        assert_eq!(
            scheduler_launch::start_from_reservation(
                &self.supervisor,
                &self.store,
                self.release.kernel(),
                &self.root.join("runtime"),
                reservation,
                &binding,
            )
            .expect("start initial Scheduler"),
            1
        );
        binding
    }

    fn persist_recovery_schedule(&self) -> i64 {
        let replaced_due_at = future_due_at_unix_millis();
        let due_at = replaced_due_at + 3_000;
        upsert_recovery_schedule(
            &self.supervisor,
            1,
            replaced_due_at,
            SchedulerScheduleUpsertOutcomeV1::Inserted,
        );
        upsert_recovery_schedule(
            &self.supervisor,
            2,
            due_at,
            SchedulerScheduleUpsertOutcomeV1::Updated,
        );
        due_at
    }

    fn restart_after_crash(&self, due_at: i64) -> std::thread::JoinHandle<Result<(), String>> {
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("simulate Scheduler crash");
        wait_until_due(due_at);
        let store = Arc::clone(&self.store);
        let supervisor = self.supervisor.clone();
        let shutdown = Arc::clone(&self.shutdown);
        let runtime_dir = self.root.join("runtime");
        let kernel = self.release.kernel().to_path_buf();
        std::thread::spawn(move || {
            scheduler_lifecycle::serve(store, &kernel, &runtime_dir, shutdown, supervisor)
        })
    }

    fn assert_successor(
        &self,
        binding: &hermes_kernel_control_store::PlatformStorageBindingV1,
        due_at: i64,
    ) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
        wait_for_scheduler_generation(&self.supervisor, &self.store, 2);
        let successor = scheduler_binding(&self.store);
        assert_eq!(successor.runtime_generation(), 2);
        assert_ne!(
            successor.runtime_instance_id(),
            binding.runtime_instance_id()
        );
        assert_eq!(successor.role_epoch(), 2);
        assert_eq!(successor.credential_lease_revision(), 2);
        assert_recovered_scheduler_delivery(&self.store, due_at);
        successor
    }

    fn assert_revoked_binding_does_not_restart(
        &self,
        successor: hermes_kernel_control_store::PlatformStorageBindingV1,
    ) {
        let revoking = self
            .store
            .begin_platform_storage_binding_revocation(
                SCHEDULER_REGISTRATION,
                STORAGE_CAPABILITY,
                successor.binding_revision(),
            )
            .expect("reserve successor binding revocation");
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("stop revoked Scheduler");
        std::thread::sleep(Duration::from_millis(600));
        assert!(
            !self
                .supervisor
                .is_active(SCHEDULER_REGISTRATION)
                .expect("read Scheduler state")
        );
        assert_eq!(revoking.runtime_generation(), 2);
    }

    fn shutdown(self, worker: std::thread::JoinHandle<Result<(), String>>) {
        self.shutdown.store(true, Ordering::Release);
        worker
            .join()
            .expect("join Scheduler lifecycle")
            .expect("lifecycle exits");
        self.supervisor.shutdown().expect("stop managed processes");
        std::fs::remove_dir_all(self.root).expect("remove fixture");
    }
}

fn assert_recovered_scheduler_delivery(store: &SqliteControlStore, due_at: i64) {
    let envelope = recovered_scheduler_delivery(store);
    assert!(
        matches!(envelope.contract, Some(contract) if contract.owner == "platform" && contract.name == "maintenance")
    );
    assert!(
        matches!(envelope.source, Some(source) if source.module_id == SCHEDULER_REGISTRATION && source.runtime_generation == 2)
    );
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .expect("decode recovered Scheduler command");
    assert_eq!(command.schedule_revision, 2);
    assert_eq!(command.scheduled_for_unix_millis, due_at);
}

fn configure_route_handler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
) {
    let blob_session_handler = Arc::new(BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        data.to_path_buf(),
    ));
    configure_route_handlers(supervisor, store, data, blob_session_handler);
}

fn configure_route_handlers(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
    blob_session_handler: Arc<dyn ManagedRuntimeBlobSessionHandler>,
) {
    let vault_route = Arc::new(KernelManagedVaultRouteHandler::new(
        Arc::clone(store),
        data,
        Arc::new(supervisor.relay_port()),
    ));
    let vault_handler: Arc<
        dyn crate::runtime::lifecycle::control::ManagedRuntimeVaultRouteHandler,
    > = vault_route.clone();
    supervisor
        .configure_vault_route_handler(vault_handler)
        .expect("Vault route handler");
    supervisor
        .configure_provider_credential_handler(Arc::new(ProviderCredentialHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            Arc::clone(&vault_route),
        )))
        .expect("provider credential handler");
    supervisor
        .configure_owner_derived_key_handler(Arc::new(OwnerDerivedKeyHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            vault_route,
        )))
        .expect("owner-derived key handler");
    supervisor
        .configure_blob_session_handler(blob_session_handler)
        .expect("Blob session handler");
}

fn upsert_recovery_schedule(
    supervisor: &ManagedRuntimeSupervisor,
    schedule_revision: u64,
    due_at: i64,
    expected_outcome: SchedulerScheduleUpsertOutcomeV1,
) {
    let request = SchedulerRuntimeControlRequestV1 {
        operation: Some(SchedulerOperation::UpsertSchedule(
            UpsertSchedulerScheduleRequestV1 {
                schedule_id: vec![9; 16],
                schedule_revision,
                job_owner: "platform".to_owned(),
                job_name: "maintenance".to_owned(),
                job_major: 1,
                contract_name: "platform.maintenance".to_owned(),
                contract_revision: 1,
                contract_schema_sha256: vec![7; 32],
                scope_id: "recovery:opaque".to_owned(),
                concurrency_key: "recovery:opaque".to_owned(),
                enabled: true,
                policy_canonical_bytes: one_shot_recovery_policy(due_at),
                next_due_at_unix_millis: due_at,
                updated_at_unix_millis: due_at - 1_000,
            },
        )),
    };
    let response = supervisor
        .relay(SCHEDULER_REGISTRATION, request.encode_to_vec())
        .expect("persist recovery schedule through Scheduler control");
    let response = SchedulerRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode Scheduler schedule response");
    assert!(matches!(
        response.result,
        Some(SchedulerResult::UpsertSchedule(result))
            if result.schedule_revision == schedule_revision
                && result.outcome == expected_outcome as i32
    ));
    assert!(response.error_code.is_empty());
}

fn future_due_at_unix_millis() -> i64 {
    current_unix_millis() + 2_000
}

fn current_unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock after epoch")
        .as_millis()
        .try_into()
        .expect("Unix milliseconds fit i64")
}

fn wait_until_due(due_at: i64) {
    let now = current_unix_millis();
    if due_at > now {
        let delay = u64::try_from(due_at - now).expect("future due delay") + 100;
        std::thread::sleep(Duration::from_millis(delay));
    }
}

fn one_shot_recovery_policy(due_at: i64) -> Vec<u8> {
    let mut policy = Vec::with_capacity(32);
    policy.push(1); // encoding version
    policy.push(1); // trigger: at
    policy.extend_from_slice(&due_at.to_be_bytes());
    policy.push(1); // overlap: forbid
    policy.push(2); // misfire: fire once after successor recovery
    policy.extend_from_slice(&1_u16.to_be_bytes()); // retry attempts
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // retry backoff
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // command deadline
    policy.extend_from_slice(&0_u64.to_be_bytes()); // jitter
    policy
}

const SCHEDULER_REGISTRATION: &str = "scheduler_registration";
const STORAGE_CAPABILITY: &str = "storage.scheduler";
const DISPATCH_CAPABILITY: &str = "events.scheduler.dispatch";
const ACK_CAPABILITY: &str = "events.scheduler.ack";
const RESULT_CAPABILITY: &str = "events.scheduler.result";

fn scheduler_binding(
    store: &SqliteControlStore,
) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(SCHEDULER_REGISTRATION, STORAGE_CAPABILITY)
        .expect("read Scheduler Storage binding")
        .expect("Scheduler Storage binding")
}

fn wait_for_scheduler_generation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    expected_generation: u64,
) {
    // A managed child is allowed 15 seconds to announce readiness; include the
    // lifecycle poll and Storage/Vault provisioning time before declaring the
    // recovery contour failed.
    let deadline = std::time::Instant::now() + Duration::from_secs(25);
    while std::time::Instant::now() < deadline {
        let active = supervisor
            .is_active(SCHEDULER_REGISTRATION)
            .expect("read Scheduler runtime state");
        let generation = store
            .effective_managed_launch_record(SCHEDULER_REGISTRATION)
            .expect("read Scheduler launch record")
            .map(|record| record.runtime_generation());
        if active && generation == Some(expected_generation) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "Scheduler successor did not reach generation {expected_generation}: {:?}",
        supervisor.last_failure(SCHEDULER_REGISTRATION)
    );
}

struct UnauthenticatedNatsCredentialHandler {
    store: Arc<SqliteControlStore>,
}

impl UnauthenticatedNatsCredentialHandler {
    fn new(store: Arc<SqliteControlStore>) -> Self {
        Self { store }
    }
}

impl ManagedRuntimeEventCredentialHandler for UnauthenticatedNatsCredentialHandler {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        let registration = self
            .store
            .module_registration(expectation.registration_id())
            .map_err(|_| "Event registration is unavailable".to_owned())?
            .ok_or_else(|| "Event registration is unavailable".to_owned())?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let recipient = NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(
            request
                .recipient_public_key_x25519
                .as_slice()
                .try_into()
                .map_err(|_| "Scheduler Event request is invalid".to_owned())?,
        )
        .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let binding = NatsRuntimeCredentialDeliveryBindingV1::new(
            NatsRuntimeCredentialDeliveryBindingInputV1 {
                logical_owner_id: registration.owner_id().to_owned(),
                registration_id: expectation.registration_id().to_owned(),
                runtime_instance_id: expectation.runtime_instance_id().to_owned(),
                runtime_generation: expectation.runtime_generation(),
                grant_epoch: expectation.grant_epoch(),
                credential_revision: request.credential_revision,
                request_id,
                recipient_public_key: recipient,
            },
        )
        .map_err(|_| "Scheduler Event binding is invalid".to_owned())?;
        let key = KeyPair::new_user();
        let credential = RuntimeNatsJwtCredentialV1::new(
            "test-jwt".to_owned(),
            key.seed()
                .map_err(|_| "Scheduler Event key is unavailable".to_owned())?,
            key.public_key(),
            u64::MAX,
        )
        .map_err(|_| "Scheduler Event credential is invalid".to_owned())?;
        let delivery = credential
            .seal_for(&binding)
            .map_err(|_| "Scheduler Event delivery is unavailable".to_owned())?;
        let contracts = event_catalog::resolve_contracts(&*self.store)
            .map_err(|_| "test Event topology is unavailable".to_owned())?;
        let configuration = self
            .store
            .platform_event_hub_topology()
            .map_err(|_| "test Event topology is unavailable".to_owned())?
            .ok_or_else(|| "test Event topology is unavailable".to_owned())?;
        let topology = event_topology::plan(&contracts, &configuration)
            .map_err(|_| "test Event topology is unavailable".to_owned())?;
        let consumer_bindings = event_topology::managed_runtime_consumer_bindings(
            &topology,
            expectation.registration_id(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "test Event consumer binding is unavailable".to_owned())?;
        let publish_subjects = event_topology::managed_runtime_publish_subjects(
            &topology,
            expectation.registration_id(),
            expectation.grant_epoch(),
        );
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: delivery.encapped_key().to_vec(),
            ciphertext: delivery.ciphertext().to_vec(),
            tag: delivery.tag().to_vec(),
            consumer_bindings,
            publish_subjects,
        })
    }
}
fn private_directory(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).expect("private directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path
}
