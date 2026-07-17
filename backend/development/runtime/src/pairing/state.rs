use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use getrandom::fill;
use sha2::{Digest, Sha256};

const STATE_FILE: &str = "development-remote-pairing-v1.state";
const RECEIPT_FILE: &str = "development-remote-pairing-v1.receipt";
const LOCK_DIRECTORY: &str = ".development-remote-pairing-v1.lock";
const MAX_TTL_SECONDS: u64 = 900;
const MAX_RECEIPT_BYTES: u64 = 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PairingStatus {
    Active,
    Consumed,
    Expired,
}

struct PairingState {
    status: PairingStatus,
    expires_at_ms: u128,
    token_sha256: [u8; 32],
    receipt_sha256: Option<[u8; 32]>,
}

pub struct RemotePairingReceipt {
    pub owner_id: String,
    pub device_id: String,
    pub challenge: [u8; 32],
    pub device_public_key_sec1: [u8; 65],
    pub signature_raw: [u8; 64],
}

pub fn create(state_dir: &Path, ttl_seconds: u64) -> Result<String, String> {
    if ttl_seconds == 0 || ttl_seconds > MAX_TTL_SECONDS {
        return Err(format!(
            "pairing TTL must be between 1 and {MAX_TTL_SECONDS} seconds"
        ));
    }
    prepare_state_directory(state_dir)?;
    let _lock = acquire_lock(state_dir)?;
    let now = unix_time_ms()?;
    if let Some(existing) = read_state(state_dir)? {
        match existing.status {
            PairingStatus::Consumed => {
                return Err("initial enrollment is already complete".to_owned());
            }
            PairingStatus::Active if existing.expires_at_ms > now => {
                return Err("an active pairing already exists".to_owned());
            }
            PairingStatus::Active | PairingStatus::Expired => remove_expired_receipt(state_dir)?,
        }
    }
    let mut token = [0_u8; 32];
    fill(&mut token).map_err(|error| error.to_string())?;
    let expires_at_ms = now
        .checked_add(u128::from(ttl_seconds) * 1_000)
        .ok_or_else(|| "pairing expiry overflow".to_owned())?;
    write_state(
        state_dir,
        &PairingState {
            status: PairingStatus::Active,
            expires_at_ms,
            token_sha256: Sha256::digest(token).into(),
            receipt_sha256: None,
        },
    )?;
    Ok(hex(&token))
}

pub fn consume(state_dir: &Path, token_hex: &str) -> Result<(), String> {
    prepare_state_directory(state_dir)?;
    let _lock = acquire_lock(state_dir)?;
    let now = unix_time_ms()?;
    let mut state = read_state(state_dir)?.ok_or_else(|| "no pairing is available".to_owned())?;
    ensure_active_token(state_dir, &mut state, token_hex, now)?;
    state.status = PairingStatus::Consumed;
    write_state(state_dir, &state)
}

pub fn validate(state_dir: &Path, token_hex: &str) -> Result<(), String> {
    prepare_state_directory(state_dir)?;
    let _lock = acquire_lock(state_dir)?;
    let now = unix_time_ms()?;
    let mut state = read_state(state_dir)?.ok_or_else(|| "no pairing is available".to_owned())?;
    ensure_active_token(state_dir, &mut state, token_hex, now)
}

pub fn complete_with_receipt(
    state_dir: &Path,
    token_hex: &str,
    receipt: &RemotePairingReceipt,
) -> Result<(), String> {
    prepare_state_directory(state_dir)?;
    let _lock = acquire_lock(state_dir)?;
    let now = unix_time_ms()?;
    let mut state = read_state(state_dir)?.ok_or_else(|| "no pairing is available".to_owned())?;
    ensure_active_token(state_dir, &mut state, token_hex, now)?;
    let receipt_bytes = encode_receipt(receipt);
    let receipt_sha256: [u8; 32] = Sha256::digest(&receipt_bytes).into();
    write_receipt(state_dir, &receipt_bytes)?;
    state.status = PairingStatus::Consumed;
    state.receipt_sha256 = Some(receipt_sha256);
    if let Err(error) = write_state(state_dir, &state) {
        let _ = std::fs::remove_file(state_dir.join(RECEIPT_FILE));
        return Err(error);
    }
    Ok(())
}

fn ensure_active_token(
    state_dir: &Path,
    state: &mut PairingState,
    token_hex: &str,
    now: u128,
) -> Result<(), String> {
    if state.status == PairingStatus::Consumed {
        return Err("initial enrollment is already complete".to_owned());
    }
    if state.status == PairingStatus::Expired || state.expires_at_ms <= now {
        state.status = PairingStatus::Expired;
        write_state(state_dir, state)?;
        return Err("pairing token expired".to_owned());
    }
    let token = decode_token(token_hex)?;
    let token_sha256: [u8; 32] = Sha256::digest(token).into();
    if !constant_time_equal(&state.token_sha256, &token_sha256) {
        return Err("pairing token rejected".to_owned());
    }
    Ok(())
}

fn prepare_state_directory(state_dir: &Path) -> Result<(), String> {
    if !state_dir.is_absolute() {
        return Err("pairing state directory must be absolute".to_owned());
    }
    match std::fs::symlink_metadata(state_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("pairing state directory must not be a symlink".to_owned());
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err("pairing state directory must be a directory".to_owned());
        }
        Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
            return Err("pairing state directory must be owner-private".to_owned());
        }
        Ok(_) => return Ok(()),
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.to_string()),
        Err(_) => {}
    }
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(state_dir).map_err(|error| error.to_string())
}

fn acquire_lock(state_dir: &Path) -> Result<PairingLock, String> {
    let lock_path = state_dir.join(LOCK_DIRECTORY);
    match std::fs::create_dir(&lock_path) {
        Ok(()) => Ok(PairingLock(lock_path)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err("pairing operation is already in progress".to_owned())
        }
        Err(error) => Err(error.to_string()),
    }
}

struct PairingLock(PathBuf);

impl Drop for PairingLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.0);
    }
}

fn read_state(state_dir: &Path) -> Result<Option<PairingState>, String> {
    let path = state_dir.join(STATE_FILE);
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("pairing state file must not be a symlink".to_owned());
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err("pairing state file must be a regular file".to_owned());
        }
        Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
            return Err("pairing state file must be owner-private".to_owned());
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut lines = content.lines();
    if lines.next() != Some("development_remote_pairing_v1") {
        return Err("pairing state is invalid".to_owned());
    }
    let status = match lines.next() {
        Some("active") => PairingStatus::Active,
        Some("consumed") => PairingStatus::Consumed,
        Some("expired") => PairingStatus::Expired,
        _ => return Err("pairing state is invalid".to_owned()),
    };
    let expires_at_ms = lines
        .next()
        .and_then(|value| value.parse::<u128>().ok())
        .ok_or_else(|| "pairing state is invalid".to_owned())?;
    let token_sha256 = lines
        .next()
        .map(decode_digest)
        .transpose()?
        .ok_or_else(|| "pairing state is invalid".to_owned())?;
    let receipt_sha256 = match lines.next() {
        None | Some("-") => None,
        Some(value) => Some(decode_digest(value)?),
    };
    if lines.next().is_some() {
        return Err("pairing state is invalid".to_owned());
    }
    Ok(Some(PairingState {
        status,
        expires_at_ms,
        token_sha256,
        receipt_sha256,
    }))
}

fn write_state(state_dir: &Path, state: &PairingState) -> Result<(), String> {
    let path = state_dir.join(STATE_FILE);
    let temporary = state_dir.join(format!(".{STATE_FILE}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let status = match state.status {
        PairingStatus::Active => "active",
        PairingStatus::Consumed => "consumed",
        PairingStatus::Expired => "expired",
    };
    let content = format!(
        "development_remote_pairing_v1\n{status}\n{}\n{}\n",
        state.expires_at_ms,
        hex(&state.token_sha256),
    );
    let content = format!(
        "{content}{}\n",
        state
            .receipt_sha256
            .map_or_else(|| "-".to_owned(), |digest| hex(&digest)),
    );
    let result = file
        .write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string());
    drop(file);
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

fn write_receipt(state_dir: &Path, receipt: &[u8]) -> Result<(), String> {
    let path = state_dir.join(RECEIPT_FILE);
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("pairing receipt must not be a symlink".to_owned());
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err("pairing receipt must be a regular file".to_owned());
        }
        Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
            return Err("pairing receipt must be owner-private".to_owned());
        }
        Ok(metadata) if metadata.len() > MAX_RECEIPT_BYTES => {
            return Err("pairing receipt is too large".to_owned());
        }
        Ok(_) => return existing_receipt_matches(&path, receipt),
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.to_string()),
        Err(_) => {}
    }
    let temporary = state_dir.join(format!(".{RECEIPT_FILE}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let result = file
        .write_all(receipt)
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string());
    drop(file);
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

fn existing_receipt_matches(path: &Path, expected: &[u8]) -> Result<(), String> {
    let observed = std::fs::read(path).map_err(|error| error.to_string())?;
    if observed == expected {
        Ok(())
    } else {
        Err("pairing receipt does not match the active enrollment".to_owned())
    }
}

fn remove_expired_receipt(state_dir: &Path) -> Result<(), String> {
    let path = state_dir.join(RECEIPT_FILE);
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err("expired pairing receipt must be a regular file".to_owned());
        }
        Ok(metadata) if metadata.permissions().mode() & 0o077 != 0 => {
            return Err("expired pairing receipt must be owner-private".to_owned());
        }
        Ok(metadata) if metadata.len() > MAX_RECEIPT_BYTES => {
            return Err("expired pairing receipt is too large".to_owned());
        }
        Ok(_) => {}
    }
    std::fs::remove_file(path).map_err(|error| error.to_string())?;
    File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

fn encode_receipt(receipt: &RemotePairingReceipt) -> Vec<u8> {
    format!(
        "development_remote_pairing_receipt_v1\n{}\n{}\n{}\n{}\n{}\n",
        receipt.owner_id,
        receipt.device_id,
        hex(&receipt.challenge),
        hex(&receipt.device_public_key_sec1),
        hex(&receipt.signature_raw),
    )
    .into_bytes()
}

fn decode_token(value: &str) -> Result<[u8; 32], String> {
    decode_digest(value).map_err(|_| "pairing token rejected".to_owned())
}

fn decode_digest(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("invalid digest".to_owned());
    }
    let mut digest = [0_u8; 32];
    for (index, output) in digest.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "invalid digest".to_owned())?;
    }
    Ok(digest)
}

fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

fn unix_time_ms() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|_| "system clock is before Unix epoch".to_owned())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
