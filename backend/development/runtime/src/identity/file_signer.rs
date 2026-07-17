use std::fs::File;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};

const DEVICE_KEY_FILE: &str = "device-es256.key";

pub struct FileDeviceSigner {
    signing_key: SigningKey,
}

impl FileDeviceSigner {
    pub fn open_or_create(key_dir: &Path) -> Result<Self, String> {
        prepare_key_directory(key_dir)?;
        let key_path = key_dir.join(DEVICE_KEY_FILE);
        match std::fs::symlink_metadata(&key_path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("device key must not be a symlink".to_owned());
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err("device key must be a regular file".to_owned());
            }
            Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
                return Err("device key must be owner-private".to_owned());
            }
            Ok(_) => {
                let bytes = std::fs::read(&key_path).map_err(|error| error.to_string())?;
                let bytes: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| "device key has an invalid length".to_owned())?;
                let signing_key = SigningKey::from_bytes((&bytes).into())
                    .map_err(|_| "device key is invalid".to_owned())?;
                return Ok(Self { signing_key });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }

        let mut secret_bytes = [0_u8; 32];
        let signing_key = loop {
            getrandom::fill(&mut secret_bytes).map_err(|error| error.to_string())?;
            if let Ok(key) = SigningKey::from_bytes((&secret_bytes).into()) {
                break key;
            }
        };
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&key_path)
            .map_err(|error| error.to_string())?;
        file.write_all(&signing_key.to_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|error| error.to_string())?;
        Ok(Self { signing_key })
    }

    pub fn public_key_sec1(&self) -> [u8; 65] {
        self.signing_key
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .expect("P-256 uncompressed public key has a fixed 65-byte encoding")
    }

    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes().into()
    }
}

fn prepare_key_directory(key_dir: &Path) -> Result<(), String> {
    if !key_dir.is_absolute() {
        return Err("device key directory must be absolute".to_owned());
    }
    match std::fs::symlink_metadata(key_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("device key directory must not be a symlink".to_owned());
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err("device key directory must be a directory".to_owned());
        }
        Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
            return Err("device key directory must be owner-private".to_owned());
        }
        Ok(_) => return Ok(()),
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.to_string()),
        Err(_) => {}
    }
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(key_dir).map_err(|error| error.to_string())
}
