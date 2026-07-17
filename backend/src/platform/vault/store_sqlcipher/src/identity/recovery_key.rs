//! One-time BIP-39 representation for the non-persistent Vault recovery key.

use bip39::{Language, Mnemonic};
use getrandom::fill;
use zeroize::Zeroizing;

const RECOVERY_KEY_BYTES: usize = 32;
const BIP39_WORD_COUNT: usize = 24;

/// A 256-bit recovery wrapping key that is never serialized by Vault.
pub struct VaultRecoveryKeyV1(Zeroizing<[u8; RECOVERY_KEY_BYTES]>);

impl VaultRecoveryKeyV1 {
    /// Creates random entropy for an explicitly requested recovery ceremony.
    pub fn generate() -> Result<Self, VaultRecoveryKeyError> {
        let mut entropy = Zeroizing::new([0; RECOVERY_KEY_BYTES]);
        fill(entropy.as_mut()).map_err(|_| VaultRecoveryKeyError::Randomness)?;
        Ok(Self(entropy))
    }

    /// Imports exactly one English 24-word BIP-39 entropy mnemonic.
    pub fn from_mnemonic(words: &str) -> Result<Self, VaultRecoveryKeyError> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, words)
            .map_err(|_| VaultRecoveryKeyError::InvalidMnemonic)?;
        if mnemonic.word_count() != BIP39_WORD_COUNT {
            return Err(VaultRecoveryKeyError::InvalidMnemonic);
        }
        let entropy = mnemonic.to_entropy();
        let entropy: [u8; RECOVERY_KEY_BYTES] = entropy
            .try_into()
            .map_err(|_| VaultRecoveryKeyError::InvalidMnemonic)?;
        Ok(Self(Zeroizing::new(entropy)))
    }

    /// Consumes the key to produce its one-time English BIP-39 display form.
    pub fn into_mnemonic(self) -> Result<String, VaultRecoveryKeyError> {
        Mnemonic::from_entropy(self.0.as_ref())
            .map(|mnemonic| mnemonic.to_string())
            .map_err(|_| VaultRecoveryKeyError::InvalidMnemonic)
    }

    pub(crate) fn as_bytes(&self) -> &[u8; RECOVERY_KEY_BYTES] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultRecoveryKeyError {
    Randomness,
    InvalidMnemonic,
}
