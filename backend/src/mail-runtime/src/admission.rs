//! Exact, still-unadmitted descriptor for the Mail integration runtime.
//!
//! This artifact describes the smallest Mail-owned capability set. It does
//! not register Mail in the production inventory or grant any capability.

use hermes_communications_ingress::admission::{
    COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
    communication_attachment_anchor_recorded_contract_reference_v1,
    communication_attachment_blob_admission_observed_publish_request_v1,
    communication_observed_publish_request_v1,
};
use hermes_runtime_protocol::v1::{
    BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
    DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    MAIL_SETTINGS_SCHEMA_MAJOR_V1, MAIL_SETTINGS_SCHEMA_REVISION_V1, mail_settings_schema_bytes_v1,
};

pub const MAIL_MODULE_ID: &str = "hermes-mail-runtime";
pub const MAIL_OWNER_ID: &str = "mail";
pub const MAIL_BLOB_CAPABILITY_ID: &str = "mail.blob.v1";
pub const MAIL_CREDENTIALS_CAPABILITY_ID: &str = "mail.credentials.v1";
pub const MAIL_EVENTS_CAPABILITY_ID: &str = "mail.events.v1";
pub const MAIL_STORAGE_CAPABILITY_ID: &str = "mail.storage.v1";
pub const MAIL_ATTACHMENT_BLOB_MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const MAIL_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const MAIL_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const MAIL_EVENT_MAX_DELIVER: u32 = 8;
pub const MAIL_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;
pub const MAIL_CREDENTIAL_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn mail_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        mail_blob_capability_v1(),
        mail_credentials_capability_v1(),
        mail_events_capability_v1(),
        mail_storage_capability_v1(),
    ]
}

#[must_use]
pub fn mail_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: MAIL_ATTACHMENT_BLOB_MAX_BYTES,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_credentials_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_CREDENTIALS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: [
            "mail_gmail_access_token",
            "mail_imap_password",
            "mail_smtp_password",
        ]
        .into_iter()
        .map(provider_credential_request_v1)
        .collect(),
        ..Default::default()
    }
}

fn provider_credential_request_v1(purpose_id: &str) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
            purpose_id: purpose_id.to_owned(),
            requested_lease_ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
            allowed_secret_classes: vec![VaultSecretClassV1::ProviderCredential as i32],
            actions: vec![VaultActionV1::Resolve as i32],
            target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
            key_schema_revision: 0,
        })),
    }
}

#[must_use]
pub fn mail_events_capability_v1() -> CapabilityDescriptorV1 {
    let anchor_recorded = communication_attachment_anchor_recorded_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: MAIL_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            communication_observed_publish_request_v1(),
            communication_attachment_blob_admission_observed_publish_request_v1(),
            CapabilityRequestV1 {
                request: Some(Request::EventRoute(EventRouteRequestV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    contract: Some(anchor_recorded),
                    direction: EventRouteDirectionV1::Consume as i32,
                    max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                    subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                    max_deliver: MAIL_EVENT_MAX_DELIVER,
                    ack_wait_millis: MAIL_EVENT_ACK_WAIT_MILLIS,
                })),
            },
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: MAIL_OWNER_ID.to_owned(),
                connection_budget: MAIL_STORAGE_CONNECTION_BUDGET,
                timeout_millis: MAIL_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = mail_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: MAIL_MODULE_ID.to_owned(),
        owner_id: MAIL_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: mail_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: MAIL_SETTINGS_SCHEMA_MAJOR_V1,
            revision: MAIL_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: MAIL_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 256 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Mail".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    use super::*;

    #[test]
    fn mail_descriptor_is_valid_and_requests_only_its_exact_boundary() {
        let descriptor = mail_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                MAIL_BLOB_CAPABILITY_ID,
                MAIL_CREDENTIALS_CAPABILITY_ID,
                MAIL_EVENTS_CAPABILITY_ID,
                MAIL_STORAGE_CAPABILITY_ID,
            ]
        );

        let events = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == MAIL_EVENTS_CAPABILITY_ID)
            .expect("mail events capability");
        assert_eq!(events.provides, []);
        assert_eq!(events.requests.len(), 3);
        assert!(
            events
                .requests
                .iter()
                .all(|request| matches!(request.request, Some(Request::EventRoute(_))))
        );
    }
}
