#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryFences {
    generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
}

impl RecoveryFences {
    #[must_use]
    pub fn new(generation: u64, identity_epoch: u64, grant_epoch: u64) -> Self {
        Self {
            generation,
            identity_epoch,
            grant_epoch,
        }
    }

    #[must_use]
    pub fn generation(self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn identity_epoch(self) -> u64 {
        self.identity_epoch
    }

    #[must_use]
    pub fn grant_epoch(self) -> u64 {
        self.grant_epoch
    }
}
