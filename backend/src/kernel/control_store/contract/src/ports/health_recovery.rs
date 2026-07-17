use crate::ControlStore;

pub trait HealthRecoveryStore {
    fn control_store_snapshot(&self) -> &ControlStore;
}
