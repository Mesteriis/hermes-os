#[cfg(test)]
mod distribution {
    #[path = "../../../../../src/kernel/src/distribution/bundle_verifier.rs"]
    pub(crate) mod bundle_verifier;
    #[path = "../../../../../src/kernel/src/distribution/bundled_launch.rs"]
    pub(crate) mod bundled_launch;
    #[path = "../../../../../src/kernel/src/distribution/manifest_verifier.rs"]
    pub(crate) mod manifest_verifier;
    #[path = "../../../../../src/kernel/src/distribution/staged_artifact.rs"]
    pub(crate) mod staged_artifact;
    #[path = "../../../../../src/kernel/src/distribution/staged_contracts.rs"]
    pub(crate) mod staged_contracts;
    #[path = "../../../../../src/kernel/src/distribution/trust_root.rs"]
    pub(crate) mod trust_root;
}

#[cfg(test)]
mod identity {
    #[path = "../../../../../src/kernel/src/identity/browser_gateway.rs"]
    pub(crate) mod browser_gateway;
    pub(crate) mod device;
    pub(crate) mod owner;
}

#[cfg(test)]
mod infrastructure {
    #[path = "../../../../../src/kernel/src/infrastructure/filesystem.rs"]
    pub(crate) mod filesystem;
}

#[cfg(test)]
mod modules {
    pub(crate) mod capability;
    pub(crate) mod registration;
}

#[cfg(test)]
mod platform {
    pub(crate) mod blob;

    #[path = "../../../../../src/kernel/src/platform/gateway.rs"]
    pub(crate) mod gateway;

    pub(crate) mod managed;
    #[path = "../../../../../src/kernel/src/platform/scheduler/mod.rs"]
    pub(crate) mod scheduler;
    #[path = "../../../../../src/kernel/src/platform/scheduler/admission.rs"]
    pub(crate) mod scheduler_admission;
    #[path = "../../../../../src/kernel/src/platform/scheduler/catalog.rs"]
    pub(crate) mod scheduler_catalog;
    #[path = "../../../../../src/kernel/src/platform/scheduler/launch.rs"]
    pub(crate) mod scheduler_launch;
    #[path = "../../../../../src/kernel/src/platform/scheduler/restart.rs"]
    pub(crate) mod scheduler_restart;
    pub(crate) use scheduler_launch as launch;
    pub(crate) use scheduler_restart as restart;
    #[path = "../../../../../src/kernel/src/platform/scheduler/lifecycle.rs"]
    pub(crate) mod scheduler_lifecycle;
    pub(crate) mod macos {
        #[path = "../../../../../../src/kernel/src/platform/macos/code_signature.rs"]
        pub(crate) mod code_signature;
        #[path = "../../../../../../src/kernel/src/platform/macos/managed_launch.rs"]
        pub(crate) mod managed_launch;
        #[path = "../../../../../../src/kernel/src/platform/macos/native_launch.rs"]
        pub(crate) mod native_launch;
        #[path = "../../../../../../src/kernel/src/platform/macos/release_resources.rs"]
        pub(crate) mod release_resources;
    }

    pub(crate) mod telemetry;

    pub(crate) mod events;
    pub(crate) mod storage;
    pub(crate) mod vault {
        #[path = "../../../../../../src/kernel/src/platform/vault/binding.rs"]
        pub(crate) mod binding;
        #[path = "../../../../../../src/kernel/src/platform/vault/ciphertext_route.rs"]
        pub(crate) mod ciphertext_route;
        #[path = "../../../../../../src/kernel/src/platform/vault/launch.rs"]
        pub(crate) mod launch;
        #[path = "../../../../../../src/kernel/src/platform/vault/managed_route.rs"]
        pub(crate) mod managed_route;
        #[path = "../../../../../../src/kernel/src/platform/vault/status.rs"]
        pub(crate) mod status;
    }
}

#[cfg(test)]
mod control;

#[cfg(test)]
mod service;

#[cfg(test)]
mod transport;

#[cfg(test)]
#[path = "../../../../src/platform/storage/vault/src/lib.rs"]
pub(crate) mod vault;

#[cfg(test)]
mod storage_control;

#[cfg(test)]
mod recovery {
    #[path = "../../../../../src/kernel/src/recovery/fence.rs"]
    pub(crate) mod fence;
}

#[cfg(test)]
mod runtime {
    pub(crate) mod external;
    pub(crate) mod lifecycle;
    pub(crate) mod managed;
}

#[cfg(test)]
mod tests {
    mod actor;
    mod blob_requests;
    mod blob_service;
    mod browser_device_identity;
    mod browser_gateway_session;
    mod common;
    mod deployment_contract;
    mod descriptor_basics;
    mod distribution_bundle_fixture;
    mod event_hub_topology_configuration;
    mod event_requests;
    mod event_topology;
    mod events_authority_account_jwt;
    mod events_authority_configuration;
    mod events_authority_launch;
    mod events_authority_managed_launch;
    mod events_authority_vault;
    mod external_storage_vault;
    mod external_storage_vault_process;
    mod gateway_http3;
    mod gateway_runtime;
    mod managed_event_credential;
    mod managed_runtime_supervision;
    mod managed_storage_vault_docker;
    mod managed_vault_binary;
    mod managed_vault_route;
    mod module_grant_snapshot;
    mod part_01;
    mod part_02;
    mod part_03;
    mod part_04;
    mod part_06;
    mod part_07;
    mod platform_vault;
    mod protocol_validation;
    mod recovery_fence;
    mod scheduler_lifecycle;
    mod scheduler_requests;
    mod settings_contract;
    mod storage_authorization;
    mod storage_launch;
    mod storage_requests;
    mod storage_status;
    mod storage_topology;
    mod storage_vault_composition;
    mod telemetry_launch;
    mod vault_route_contract;
    mod vault_status;
}
#[cfg(test)]
pub(crate) use hermes_kernel_control_store_sqlite::StoreError;

#[cfg(test)]
#[path = "../../../../src/kernel/control_store/sqlite/src/actor/handle.rs"]
mod control_store_handle;
