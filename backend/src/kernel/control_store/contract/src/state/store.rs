#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreHealth {
    Trustworthy,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlStore {
    instance_id: String,
    generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
    health: StoreHealth,
}

impl ControlStore {
    #[must_use]
    pub fn new(instance_id: impl Into<String>, generation: u64) -> Self {
        Self {
            instance_id: instance_id.into(),
            generation,
            identity_epoch: 1,
            grant_epoch: 1,
            health: StoreHealth::Trustworthy,
        }
    }
    #[must_use]
    pub fn with_recovery_fences(
        instance_id: impl Into<String>,
        generation: u64,
        identity_epoch: u64,
        grant_epoch: u64,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            generation,
            identity_epoch,
            grant_epoch,
            health: StoreHealth::Trustworthy,
        }
    }
    #[must_use]
    pub fn health(&self) -> StoreHealth {
        self.health
    }
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }
    #[must_use]
    pub fn identity_epoch(&self) -> u64 {
        self.identity_epoch
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
