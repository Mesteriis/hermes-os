//! Vault-only HPKE receiver that owns the ephemeral private key.

use hermes_vault_protocol::{
    VaultCiphertextFrameV1, VaultTransportBindingV1, VaultTransportError, VaultTransportPublicKey,
};
use hpke::{
    Deserializable, Kem as KemTrait, OpModeR, Serializable,
    aead::{AeadTag, ChaCha20Poly1305},
    inout::InOutBuf,
    kdf::HkdfSha256,
    kem::X25519HkdfSha256,
};
use zeroize::Zeroizing;

const TRANSPORT_INFO: &[u8] = b"hermes-vault/hpke/v1";

type Kem = X25519HkdfSha256;
type Aead = ChaCha20Poly1305;
type Kdf = HkdfSha256;

pub struct VaultTransportKeyPair {
    private_key: <Kem as KemTrait>::PrivateKey,
    public_key: VaultTransportPublicKey,
}

impl VaultTransportKeyPair {
    #[must_use]
    pub fn generate() -> Self {
        let (private_key, public_key) = Kem::gen_keypair();
        let bytes = public_key.to_bytes();
        let bytes = bytes
            .as_slice()
            .try_into()
            .expect("X25519 public key has a fixed size");
        Self {
            private_key,
            public_key: VaultTransportPublicKey::from_bytes(bytes)
                .expect("generated X25519 key is valid"),
        }
    }

    #[must_use]
    pub fn public_key(&self) -> &VaultTransportPublicKey {
        &self.public_key
    }

    pub fn open(
        &self,
        binding: &VaultTransportBindingV1,
        frame: &VaultCiphertextFrameV1,
    ) -> Result<Zeroizing<Vec<u8>>, VaultTransportError> {
        let encapped_key = <Kem as KemTrait>::EncappedKey::from_bytes(frame.encapped_key())
            .map_err(|_| VaultTransportError::MalformedFrame)?;
        let tag = AeadTag::<Aead>::from_bytes(frame.tag())
            .map_err(|_| VaultTransportError::MalformedFrame)?;
        let mut receiver = hpke::setup_receiver::<Aead, Kdf, Kem>(
            &OpModeR::Base,
            &self.private_key,
            &encapped_key,
            TRANSPORT_INFO,
        )
        .map_err(|_| VaultTransportError::MalformedFrame)?;
        let mut plaintext = frame.ciphertext().to_vec();
        receiver
            .open_inout_detached(
                InOutBuf::from(plaintext.as_mut_slice()),
                &binding.associated_data(),
                &tag,
            )
            .map_err(|_| VaultTransportError::AuthenticationFailed)?;
        Ok(Zeroizing::new(plaintext))
    }
}
