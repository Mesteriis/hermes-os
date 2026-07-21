//! Capture command path for owner-authorized whole-instance recovery media.

use std::io::Write;
use std::path::{Path, PathBuf};

use p256::ecdsa::{Signature, SigningKey, signature::Signer};

use crate::cli::WholeInstanceCaptureCli;
use crate::identity::owner::authorization::operation_digest;
use crate::recovery::{
    capture_coordinator::capture_verified_instance,
    media::{
        encryption::RecoveryMediaEncryptionKey,
        format::{RecoveryMediaInventoryV1, RecoveryMediaProvenanceV1},
        publish::RecoveryMediaSigner,
    },
    process_port::{PostgresRecoveryCommandV1, ProcessWholeInstanceCapturePort},
};

use super::keys::read_secret_key;
use super::staging::StagedRecoveryComponents;

pub(crate) fn capture_instance(
    data_dir: Option<PathBuf>,
    capture: WholeInstanceCaptureCli,
) -> Result<(), String> {
    let data_dir = explicit_data_dir(data_dir)?;
    validate_capture_inputs(&capture)?;
    confirm_capture(&data_dir, &capture.destination)?;
    let staged = StagedRecoveryComponents::prepare(
        &data_dir,
        capture.include_blob,
        capture.include_scheduler,
    )?;
    let result = capture_after_staging(&data_dir, &capture, &staged);
    let cleanup = staged.remove();
    cleanup?;
    result.map(|published| {
        println!("whole_instance_media={}", published.display());
    })
}

fn capture_after_staging(
    data_dir: &Path,
    capture: &WholeInstanceCaptureCli,
    staged: &StagedRecoveryComponents,
) -> Result<PathBuf, String> {
    let mut port = ProcessWholeInstanceCapturePort::open(
        data_dir.to_owned(),
        staged.components(),
        postgres_command(capture)?,
    )?;
    port.authorize_owner(capture_authorization_digest(capture)?)?;
    let signer =
        FileRecoveryMediaSigner::open(&capture.media_key_id, &capture.media_signing_key_file)?;
    let encryption_key =
        RecoveryMediaEncryptionKey::new(read_secret_key(&capture.media_encryption_key_file)?);
    let provenance = RecoveryMediaProvenanceV1::new(
        port.backup_generation(),
        capture.source_commit.clone(),
        parse_digest(&capture.cargo_lock_sha256, "Cargo lock")?,
        parse_digest(&capture.toolchain_sha256, "toolchain")?,
        parse_digest(&capture.policy_sha256, "policy")?,
    )?;
    capture_verified_instance(
        &capture.destination,
        provenance,
        RecoveryMediaInventoryV1::new(capture.include_blob, capture.include_scheduler),
        &signer,
        &encryption_key,
        &mut port,
    )
}

fn explicit_data_dir(data_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    data_dir
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "whole-instance capture requires an explicit absolute --data-dir".to_owned())
}

fn validate_capture_inputs(capture: &WholeInstanceCaptureCli) -> Result<(), String> {
    for path in [
        &capture.destination,
        &capture.media_encryption_key_file,
        &capture.media_signing_key_file,
        &capture.pg_dump,
        &capture.pg_restore,
        &capture.psql,
        &capture.postgres_password_file,
    ] {
        if !path.is_absolute() {
            return Err("whole-instance capture paths must be absolute".to_owned());
        }
    }
    if capture.media_key_id.is_empty()
        || capture.media_key_id.len() > 128
        || !capture.media_key_id.is_ascii()
    {
        return Err("recovery media key ID is invalid".to_owned());
    }
    for value in [
        &capture.postgres_host,
        &capture.postgres_database,
        &capture.postgres_username,
        &capture.postgres_ssl_mode,
    ] {
        if value.is_empty() || value.contains('\0') {
            return Err("PostgreSQL recovery connection input is invalid".to_owned());
        }
    }
    Ok(())
}

fn confirm_capture(data_dir: &Path, destination: &Path) -> Result<(), String> {
    eprintln!("offline_whole_instance_capture=true");
    eprintln!("source_data_dir={}", data_dir.display());
    eprintln!("media_destination={}", destination.display());
    eprint!("Type CAPTURE to confirm: ");
    std::io::stderr()
        .flush()
        .map_err(|error| error.to_string())?;
    let mut confirmation = String::new();
    std::io::stdin()
        .read_line(&mut confirmation)
        .map_err(|error| error.to_string())?;
    (confirmation.trim() == "CAPTURE")
        .then_some(())
        .ok_or_else(|| "whole-instance capture was not confirmed".to_owned())
}

fn postgres_command(
    capture: &WholeInstanceCaptureCli,
) -> Result<PostgresRecoveryCommandV1, String> {
    Ok(PostgresRecoveryCommandV1 {
        pg_dump: capture.pg_dump.clone(),
        pg_restore: capture.pg_restore.clone(),
        psql: capture.psql.clone(),
        host: capture.postgres_host.clone(),
        port: capture.postgres_port,
        database: capture.postgres_database.clone(),
        username: capture.postgres_username.clone(),
        ssl_mode: capture.postgres_ssl_mode.clone(),
        password_file: capture.postgres_password_file.clone(),
    })
}

fn capture_authorization_digest(capture: &WholeInstanceCaptureCli) -> Result<[u8; 32], String> {
    let destination = capture
        .destination
        .to_str()
        .ok_or_else(|| "recovery destination is not valid UTF-8".to_owned())?;
    operation_digest(&[
        destination,
        &capture.media_key_id,
        &capture.source_commit,
        &capture.cargo_lock_sha256,
        &capture.toolchain_sha256,
        &capture.policy_sha256,
    ])
}

fn parse_digest(value: &str, label: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{label} digest is invalid"));
    }
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("{label} digest is invalid"))?;
    }
    Ok(digest)
}

struct FileRecoveryMediaSigner {
    key_id: String,
    signing_key: SigningKey,
}

impl FileRecoveryMediaSigner {
    fn open(key_id: &str, path: &Path) -> Result<Self, String> {
        let signing_key = SigningKey::from_bytes((&read_secret_key(path)?).into())
            .map_err(|_| "recovery media signing key is invalid".to_owned())?;
        Ok(Self {
            key_id: key_id.to_owned(),
            signing_key,
        })
    }
}

impl RecoveryMediaSigner for FileRecoveryMediaSigner {
    fn key_id(&self) -> &str {
        &self.key_id
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        let signature: Signature = self.signing_key.sign(manifest);
        Ok(signature.to_bytes().into())
    }
}
