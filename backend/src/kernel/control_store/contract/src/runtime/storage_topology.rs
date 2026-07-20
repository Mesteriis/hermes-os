//! Desired non-secret Storage infrastructure topology owned by Kernel policy.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageDeploymentProfileV1 {
    MacosTauriEmbedded,
    LinuxDockerServer,
}

impl StorageDeploymentProfileV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MacosTauriEmbedded => "macos_tauri_embedded",
            Self::LinuxDockerServer => "linux_docker_server",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "macos_tauri_embedded" => Some(Self::MacosTauriEmbedded),
            "linux_docker_server" => Some(Self::LinuxDockerServer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStorageEndpointV1 {
    host: String,
    port: u16,
}

impl PlatformStorageEndpointV1 {
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStorageTopology {
    revision: u64,
    storage_generation: u64,
    storage_instance_id: String,
    database_id: String,
    deployment_profile: StorageDeploymentProfileV1,
    postgres_endpoint: PlatformStorageEndpointV1,
    pgbouncer_backend_endpoint: PlatformStorageEndpointV1,
    pgbouncer_endpoint: PlatformStorageEndpointV1,
    postgres_artifact_sha256: [u8; 32],
    pgbouncer_artifact_sha256: [u8; 32],
}

impl PlatformStorageTopology {
    #[must_use]
    pub fn new(
        revision: u64,
        storage_generation: u64,
        storage_instance_id: impl Into<String>,
        database_id: impl Into<String>,
        deployment_profile: StorageDeploymentProfileV1,
        postgres_endpoint: PlatformStorageEndpointV1,
        pgbouncer_endpoint: PlatformStorageEndpointV1,
        postgres_artifact_sha256: [u8; 32],
        pgbouncer_artifact_sha256: [u8; 32],
    ) -> Self {
        Self {
            revision,
            storage_generation,
            storage_instance_id: storage_instance_id.into(),
            database_id: database_id.into(),
            deployment_profile,
            pgbouncer_backend_endpoint: postgres_endpoint.clone(),
            postgres_endpoint,
            pgbouncer_endpoint,
            postgres_artifact_sha256,
            pgbouncer_artifact_sha256,
        }
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn storage_generation(&self) -> u64 {
        self.storage_generation
    }

    #[must_use]
    pub fn storage_instance_id(&self) -> &str {
        &self.storage_instance_id
    }

    #[must_use]
    pub fn database_id(&self) -> &str {
        &self.database_id
    }

    #[must_use]
    pub const fn deployment_profile(&self) -> StorageDeploymentProfileV1 {
        self.deployment_profile
    }

    #[must_use]
    pub fn postgres_endpoint(&self) -> &PlatformStorageEndpointV1 {
        &self.postgres_endpoint
    }

    /// PostgreSQL address resolved by the PgBouncer process itself.
    #[must_use]
    pub fn pgbouncer_backend_endpoint(&self) -> &PlatformStorageEndpointV1 {
        &self.pgbouncer_backend_endpoint
    }

    #[must_use]
    pub fn with_pgbouncer_backend_endpoint(mut self, endpoint: PlatformStorageEndpointV1) -> Self {
        self.pgbouncer_backend_endpoint = endpoint;
        self
    }

    #[must_use]
    pub fn pgbouncer_endpoint(&self) -> &PlatformStorageEndpointV1 {
        &self.pgbouncer_endpoint
    }

    #[must_use]
    pub const fn postgres_artifact_sha256(&self) -> &[u8; 32] {
        &self.postgres_artifact_sha256
    }

    #[must_use]
    pub const fn pgbouncer_artifact_sha256(&self) -> &[u8; 32] {
        &self.pgbouncer_artifact_sha256
    }
}
