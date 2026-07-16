use std::fs::File;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use hermes_kernel_control_store::{
    ExternalRuntimeAttestation, InitialOwnerIdentity, ModuleRegistration, ModuleRegistrationState,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::{
    descriptor_validation::{decode_descriptor_v1, validate_initial_owner_enrollment},
    v1::{InitialOwnerEnrollmentChallengeV1, InitialOwnerEnrollmentV1},
};
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::control_store_lifecycle::{bootstrap_control_store, open_validated_control_store};
use crate::filesystem::{
    acquire_runtime_directory_lock, ensure_not_symlink, ensure_owner_private_directory,
    ensure_regular_file_or_absent, new_instance_id, resolve_data_directory,
    resolve_runtime_directory,
};

const DEVELOPMENT_DEVICE_KEY_FILE: &str = "development-device-es256.key";
const INITIAL_ENROLLMENT_DOMAIN: &[u8] = b"hermes.initial-owner-enrollment.v1\0";

pub fn run_initial_owner_enrollment(
    data_dir_override: Option<PathBuf>,
    owner_id: &str,
    device_id: &str,
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 uses a software development signer and must not receive production secrets or private user data");
    if !valid_development_identity(owner_id) || !valid_development_identity(device_id) {
        return Err("development owner_id and device_id must be ASCII identifiers".to_owned());
    }
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir_existed = data_dir.exists();
    ensure_not_symlink(&data_dir, "data directory")?;
    std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    ensure_not_symlink(&data_dir, "data directory")?;
    if data_dir_existed {
        ensure_owner_private_directory(&data_dir)?;
    } else {
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");
    let store = bootstrap_control_store(&data_dir, &store_path)?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_some()
    {
        return Err("initial owner is already enrolled for this instance".to_owned());
    }

    let instance_id = store.snapshot().instance_id().as_bytes().to_vec();
    let mut nonce = [0_u8; 32];
    getrandom::fill(&mut nonce).map_err(|error| error.to_string())?;
    let challenge = InitialOwnerEnrollmentChallengeV1 {
        protocol_major: 1,
        instance_id,
        nonce: nonce.to_vec(),
        kernel_generation: store.snapshot().generation(),
    };
    let signing_key = create_development_signing_key(&data_dir)?;
    let public_key = signing_key.verifying_key().to_sec1_point(false);
    let proof_message = initial_enrollment_proof_message(&challenge);
    let signature: Signature = signing_key.sign(&proof_message);
    let enrollment = InitialOwnerEnrollmentV1 {
        protocol_major: 1,
        device_public_key_sec1: public_key.as_bytes().to_vec(),
        challenge_signature_raw: signature.to_bytes().to_vec(),
        owner_id: owner_id.to_owned(),
        device_id: device_id.to_owned(),
    };
    verify_and_claim_initial_owner(&store, &challenge, &enrollment)?;
    println!("development_initial_owner_enrolled=true");
    println!("owner_id={owner_id}");
    println!("device_id={device_id}");
    Ok(())
}

pub fn run_module_registration(
    data_dir_override: Option<PathBuf>,
    descriptor_path: &Path,
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 accepts local untrusted module descriptors");
    if !descriptor_path.is_absolute() {
        return Err("module descriptor must be an absolute path".to_owned());
    }
    ensure_regular_file_or_absent(descriptor_path, "module descriptor")?;
    if !descriptor_path.exists() {
        return Err("module descriptor does not exist".to_owned());
    }
    let descriptor_bytes = std::fs::read(descriptor_path).map_err(|error| error.to_string())?;
    let descriptor = decode_descriptor_v1(&descriptor_bytes)
        .map_err(|_| "module descriptor is invalid or exceeds protocol limits".to_owned())?;
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir_existed = data_dir.exists();
    ensure_not_symlink(&data_dir, "data directory")?;
    std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    if data_dir_existed {
        ensure_owner_private_directory(&data_dir)?;
    } else {
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store = bootstrap_control_store(&data_dir, &data_dir.join("kernel-control-store.sqlite"))?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("development module registration requires an enrolled initial owner".to_owned());
    }
    let descriptor_sha256: [u8; 32] = Sha256::digest(&descriptor_bytes).into();
    for _ in 0..16 {
        let registration = ModuleRegistration::new(
            new_instance_id()?,
            descriptor.module_id.clone(),
            descriptor.owner_id.clone(),
            descriptor_sha256,
            ModuleRegistrationState::Pending,
            1,
        );
        match store.create_pending_registration(&registration, &capability_ids) {
            Ok(()) => {
                println!("module_registration_id={}", registration.registration_id());
                println!("module_registration_state=pending");
                return Ok(());
            }
            Err(hermes_kernel_control_store_sqlite::StoreError::ModuleRegistrationAlreadyExists) => continue,
            Err(error) => return Err(format!("{error:?}")),
        }
    }
    Err("unable to allocate a unique module registration ID".to_owned())
}

pub fn run_module_approval(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 applies owner grants without a hardware-backed presence proof");
    let data_dir = resolve_data_directory(data_dir_override)?;
    if !data_dir.exists() {
        return Err("development data directory does not exist".to_owned());
    }
    ensure_not_symlink(&data_dir, "data directory")?;
    ensure_owner_private_directory(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    ensure_owner_private_directory(&runtime_dir)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store = open_validated_control_store(&data_dir.join("kernel-control-store.sqlite"))?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("development module approval requires an enrolled initial owner".to_owned());
    }
    let grants = store
        .approve_module_registration(registration_id, capability_ids)
        .map_err(|error| format!("{error:?}"))?;
    println!("module_registration_id={}", grants.registration_id());
    println!("module_grant_epoch={}", grants.grant_epoch());
    println!("effective_capability_count={}", grants.capability_ids().len());
    Ok(())
}

pub fn run_module_transition(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    state: &str,
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 applies owner lifecycle mutations without a hardware-backed presence proof");
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let next = match state {
        "suspended" => ModuleRegistrationState::Suspended,
        "revoked" => ModuleRegistrationState::Revoked,
        _ => return Err("unsupported module transition state".to_owned()),
    };
    let registration = store
        .transition_module_registration(registration_id, next)
        .map_err(|error| format!("{error:?}"))?;
    println!("module_registration_id={}", registration.registration_id());
    println!("module_registration_state={}", registration.state().as_str());
    println!("module_grant_epoch={}", registration.grant_epoch());
    Ok(())
}

pub fn run_module_status(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
) -> Result<(), String> {
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "module registration does not exist".to_owned())?;
    let effective_count = store
        .effective_grant_set(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(0, |grants| grants.capability_ids().len());
    println!("module_registration_id={}", registration.registration_id());
    println!("module_registration_state={}", registration.state().as_str());
    println!("module_grant_epoch={}", registration.grant_epoch());
    println!("effective_capability_count={effective_count}");
    if let Some(attestation) = store
        .effective_external_runtime_attestation(registration_id)
        .map_err(|error| format!("{error:?}"))?
    {
        println!("external_runtime_id={}", attestation.runtime_id());
        println!("external_runtime_generation={}", attestation.runtime_generation());
    } else {
        println!("external_runtime_attested=false");
    }
    Ok(())
}

pub fn run_external_runtime_attestation(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
    distribution_sha256: &str,
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 accepts an untrusted external runtime attestation; this does not control Docker");
    let distribution_sha256 = decode_sha256_hex(distribution_sha256)?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let grants = store
        .effective_grant_set(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "external runtime attestation requires an approved module registration".to_owned())?;
    let attestation = ExternalRuntimeAttestation::new(
        registration_id,
        runtime_id,
        runtime_generation,
        grants.grant_epoch(),
        distribution_sha256,
    );
    store
        .attest_external_runtime(&attestation)
        .map_err(|error| format!("{error:?}"))?;
    println!("module_registration_id={registration_id}");
    println!("external_runtime_id={runtime_id}");
    println!("external_runtime_generation={runtime_generation}");
    println!("external_runtime_grant_epoch={}", grants.grant_epoch());
    Ok(())
}

fn open_development_control_store(
    data_dir_override: Option<PathBuf>,
) -> Result<(PathBuf, SqliteControlStore), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    if !data_dir.exists() {
        return Err("development data directory does not exist".to_owned());
    }
    ensure_not_symlink(&data_dir, "data directory")?;
    ensure_owner_private_directory(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    ensure_owner_private_directory(&runtime_dir)?;
    let store = open_validated_control_store(&data_dir.join("kernel-control-store.sqlite"))?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("development control-plane operation requires an enrolled initial owner".to_owned());
    }
    Ok((runtime_dir, store))
}

fn valid_development_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn decode_sha256_hex(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("distribution SHA-256 must be exactly 64 hexadecimal characters".to_owned());
    }
    let mut digest = [0_u8; 32];
    for (index, output) in digest.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "distribution SHA-256 must be hexadecimal".to_owned())?;
    }
    Ok(digest)
}

fn create_development_signing_key(data_dir: &Path) -> Result<SigningKey, String> {
    let key_path = data_dir.join(DEVELOPMENT_DEVICE_KEY_FILE);
    ensure_regular_file_or_absent(&key_path, "development device key")?;
    if key_path.exists() {
        return Err("development device key already exists; explicit offline reset is required before a new initial enrollment".to_owned());
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
    Ok(signing_key)
}

fn initial_enrollment_proof_message(challenge: &InitialOwnerEnrollmentChallengeV1) -> Vec<u8> {
    let mut message =
        Vec::with_capacity(INITIAL_ENROLLMENT_DOMAIN.len() + challenge.encoded_len());
    message.extend_from_slice(INITIAL_ENROLLMENT_DOMAIN);
    challenge
        .encode(&mut message)
        .expect("Vec allocation cannot fail");
    message
}

fn verify_and_claim_initial_owner(
    store: &SqliteControlStore,
    challenge: &InitialOwnerEnrollmentChallengeV1,
    enrollment: &InitialOwnerEnrollmentV1,
) -> Result<(), String> {
    if !validate_initial_owner_enrollment(challenge, enrollment) {
        return Err("initial owner enrollment is malformed".to_owned());
    }
    let public_key: [u8; 65] = enrollment
        .device_public_key_sec1
        .clone()
        .try_into()
        .map_err(|_| "initial owner public key has an invalid length".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| "initial owner public key is invalid".to_owned())?;
    let signature = Signature::from_slice(&enrollment.challenge_signature_raw)
        .map_err(|_| "initial owner signature is invalid".to_owned())?;
    verifying_key
        .verify(&initial_enrollment_proof_message(challenge), &signature)
        .map_err(|_| "initial owner proof verification failed".to_owned())?;
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            enrollment.owner_id.clone(),
            enrollment.device_id.clone(),
            public_key,
        ))
        .map_err(|error| format!("{error:?}"))
}
