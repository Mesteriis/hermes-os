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
    pub(crate) mod device;
}

#[cfg(test)]
mod infrastructure {
    #[path = "../../../../../src/kernel/src/infrastructure/filesystem.rs"]
    pub(crate) mod filesystem;
}

#[cfg(test)]
mod modules {
    pub(crate) mod capability;
}

#[cfg(test)]
mod platform {
    pub(crate) mod macos {
        #[path = "../../../../../../src/kernel/src/platform/macos/code_signature.rs"]
        pub(crate) mod code_signature;
        #[path = "../../../../../../src/kernel/src/platform/macos/native_launch.rs"]
        pub(crate) mod native_launch;
        #[path = "../../../../../../src/kernel/src/platform/macos/release_resources.rs"]
        pub(crate) mod release_resources;
    }

    pub(crate) mod telemetry;

    pub(crate) mod vault {
        #[path = "../../../../../../src/kernel/src/platform/vault/ciphertext_route.rs"]
        pub(crate) mod ciphertext_route;
    }
}

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
    mod common;
    mod descriptor_basics;
    mod distribution_bundle_fixture;
    mod module_grant_snapshot;
    mod part_01;
    mod part_02;
    mod part_03;
    mod part_04;
    mod part_05;
    mod part_06;
    mod part_07;
    mod protocol_validation;
    mod recovery_fence;
    mod telemetry_launch;
}
#[cfg(test)]
pub(crate) use hermes_kernel_control_store_sqlite::StoreError;

#[cfg(test)]
#[path = "../../../../src/kernel/control_store/sqlite/src/actor/handle.rs"]
mod control_store_handle;
