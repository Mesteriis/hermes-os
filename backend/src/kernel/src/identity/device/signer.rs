use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};

const DEVICE_KEY_FILE: &str = "device-es256.key";

pub trait DeviceSigner {
    fn public_key_sec1(&self) -> [u8; 65];
    fn sign(&self, message: &[u8]) -> [u8; 64];
}

pub struct FileDeviceSigner {
    signing_key: SigningKey,
}

impl FileDeviceSigner {
    #[must_use]
    pub fn key_path(data_dir: &Path) -> std::path::PathBuf {
        data_dir.join(DEVICE_KEY_FILE)
    }

    pub fn open_or_create_for_instance(data_dir: &Path) -> Result<(Self, bool), String> {
        let key_path = Self::key_path(data_dir);
        ensure_regular_file_or_absent(&key_path)?;
        let mut secret_bytes = [0_u8; 32];
        let signing_key = loop {
            getrandom::fill(&mut secret_bytes).map_err(|error| error.to_string())?;
            if let Ok(key) = SigningKey::from_bytes((&secret_bytes).into()) {
                break key;
            }
        };
        let mut file = match File::options()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&key_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Self::open_for_instance(data_dir).map(|signer| (signer, false));
            }
            Err(error) => return Err(error.to_string()),
        };
        file.write_all(&signing_key.to_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|error| error.to_string())?;
        Ok((Self { signing_key }, true))
    }

    pub fn open_for_instance(data_dir: &Path) -> Result<Self, String> {
        let key_path = data_dir.join(DEVICE_KEY_FILE);
        ensure_regular_file(&key_path)?;
        let metadata = std::fs::metadata(&key_path).map_err(|error| error.to_string())?;
        if metadata.mode() & 0o077 != 0 {
            return Err("device key must not be group- or world-readable".to_owned());
        }
        if metadata.len() != 32 {
            return Err("device key has an invalid length".to_owned());
        }
        let mut secret_bytes = [0_u8; 32];
        let mut file = File::open(&key_path).map_err(|error| error.to_string())?;
        file.read_exact(&mut secret_bytes)
            .map_err(|error| error.to_string())?;
        let signing_key = SigningKey::from_bytes((&secret_bytes).into())
            .map_err(|_| "device key is invalid".to_owned())?;
        Ok(Self { signing_key })
    }
}

impl DeviceSigner for FileDeviceSigner {
    fn public_key_sec1(&self) -> [u8; 65] {
        self.signing_key
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .expect("P-256 uncompressed public key has a fixed 65-byte encoding")
    }

    fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes().into()
    }
}

fn ensure_regular_file_or_absent(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err("device key must not be a symlink".to_owned())
        }
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err("device key must be a regular file".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn ensure_regular_file(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err("device key must not be a symlink".to_owned())
        }
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err("device key must be a regular file".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err("device key does not exist".to_owned())
        }
        Err(error) => Err(error.to_string()),
    }
}
