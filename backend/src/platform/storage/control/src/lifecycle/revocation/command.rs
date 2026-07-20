//! Pool fence operations applied in strict order for one bound runtime.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoragePoolFenceCommandV1 {
    Pause,
    Disable,
    Kill,
}
