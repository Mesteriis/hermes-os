//! Opaque, encrypted Vault transport session handed across capability routing.

use crate::{VaultCiphertextFrameV1, VaultTransportBindingV1};

pub struct VaultTransportSessionV1 {
    binding: VaultTransportBindingV1,
    frame: VaultCiphertextFrameV1,
}

impl VaultTransportSessionV1 {
    #[must_use]
    pub fn new(binding: VaultTransportBindingV1, frame: VaultCiphertextFrameV1) -> Self {
        Self { binding, frame }
    }

    #[must_use]
    pub fn binding(&self) -> &VaultTransportBindingV1 {
        &self.binding
    }

    #[must_use]
    pub fn frame(&self) -> &VaultCiphertextFrameV1 {
        &self.frame
    }

    #[must_use]
    pub fn into_parts(self) -> (VaultTransportBindingV1, VaultCiphertextFrameV1) {
        (self.binding, self.frame)
    }
}
