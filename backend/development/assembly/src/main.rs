//! Owner-authorized composition of the exact local development module plan.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use hermes_gateway_protocol::owner_control_client::{
    OwnerControlClientV1, OwnerControlProofSignerV1,
};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use sha2::{Digest, Sha256};

const STATE_FILE: &str = "development-assembly-state-v1";
const ENSEMBLE_RESERVATION_FILE: &str = "development-ensemble-reservation-v2";
const DEVICE_KEY_FILE: &str = "device-es256.key";
const COMMUNICATIONS_RUNTIME_ARTIFACT: &str = "communications.runtime.v1";
const COMMUNICATIONS_STORAGE_ARTIFACT: &str = "communications.storage.v1";
const COMMUNICATIONS_STORAGE_CAPABILITY: &str = "communications.storage.v1";
const ATTACHMENT_SECURITY_RUNTIME_ARTIFACT: &str = "attachment_security.runtime.v1";
const ATTACHMENT_SECURITY_STORAGE_ARTIFACT: &str = "attachment_security.storage.v1";
const ATTACHMENT_SECURITY_STORAGE_CAPABILITY: &str = "attachment_security.storage.v1";
const MAIL_RUNTIME_ARTIFACT: &str = "mail.runtime.v1";
const MAIL_STORAGE_ARTIFACT: &str = "mail.storage.v1";
const MAIL_STORAGE_CAPABILITY: &str = "mail.storage.v1";
const TELEGRAM_RUNTIME_ARTIFACT: &str = "telegram.runtime.v1";
const TELEGRAM_STORAGE_ARTIFACT: &str = "telegram.storage.v1";
const TELEGRAM_STORAGE_CAPABILITY: &str = "telegram.storage.v1";
const WHATSAPP_RUNTIME_ARTIFACT: &str = "whatsapp.runtime.v1";
const WHATSAPP_STORAGE_ARTIFACT: &str = "whatsapp.storage.v1";
const WHATSAPP_STORAGE_CAPABILITY: &str = "whatsapp.storage.v1";
const ZULIP_RUNTIME_ARTIFACT: &str = "zulip.runtime.v1";
const ZULIP_STORAGE_ARTIFACT: &str = "zulip.storage.v1";
const ZULIP_STORAGE_CAPABILITY: &str = "zulip.storage.v1";

#[derive(Parser)]
#[command(name = "hermes-development-assembly")]
struct Cli {
    #[arg(long)]
    data_dir: PathBuf,
    #[arg(long, default_value = "hermes-local-development")]
    distribution_id: String,
    #[arg(long, default_value_t = 1)]
    distribution_generation: u64,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ProvisionPlatform,
    RuntimeDirectory,
    Admit,
    StartEnsemble,
    Status,
}

#[derive(Clone, Copy)]
enum ModuleRuntimeKindV1 {
    Domain,
    Engine,
    Integration,
}

#[derive(Clone, Copy)]
struct ModulePlanV1 {
    runtime_artifact_id: &'static str,
    storage_artifact_id: &'static str,
    storage_capability_id: &'static str,
    runtime_kind: ModuleRuntimeKindV1,
    configuration_instance_id: Option<&'static str>,
    request_host_bridge: bool,
}

const MODULE_PLAN: [ModulePlanV1; 6] = [
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATIONS_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATIONS_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATIONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        configuration_instance_id: None,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_SECURITY_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_SECURITY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        configuration_instance_id: None,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: MAIL_RUNTIME_ARTIFACT,
        storage_artifact_id: MAIL_STORAGE_ARTIFACT,
        storage_capability_id: MAIL_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        configuration_instance_id: Some("mail-development"),
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: TELEGRAM_RUNTIME_ARTIFACT,
        storage_artifact_id: TELEGRAM_STORAGE_ARTIFACT,
        storage_capability_id: TELEGRAM_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        configuration_instance_id: Some("telegram-development"),
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: WHATSAPP_RUNTIME_ARTIFACT,
        storage_artifact_id: WHATSAPP_STORAGE_ARTIFACT,
        storage_capability_id: WHATSAPP_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        configuration_instance_id: Some("whatsapp-development"),
        request_host_bridge: true,
    },
    ModulePlanV1 {
        runtime_artifact_id: ZULIP_RUNTIME_ARTIFACT,
        storage_artifact_id: ZULIP_STORAGE_ARTIFACT,
        storage_capability_id: ZULIP_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        configuration_instance_id: Some("zulip-development"),
        request_host_bridge: false,
    },
];

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("development assembly failed: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    validate_cli(&cli)?;
    let data_dir = cli
        .data_dir
        .canonicalize()
        .map_err(|_| "development data directory is unavailable".to_owned())?;
    let state_path = data_dir.join(STATE_FILE);
    match cli.command {
        Command::RuntimeDirectory => {
            println!("{}", runtime_directory(&data_dir)?.display());
            Ok(())
        }
        Command::ProvisionPlatform => {
            provision_platform(&data_dir)?;
            println!("development_platform=provisioned");
            Ok(())
        }
        Command::Status => {
            let state = read_state_if_present(&state_path)?;
            println!(
                "development_assembly={}",
                if state.is_some() {
                    "admitted"
                } else {
                    "missing"
                }
            );
            Ok(())
        }
        Command::Admit => {
            let client = client(&data_dir)?;
            let signer = FileOwnerSigner::open(&data_dir)?;
            let owner_session_id = client.open_owner_session(&signer)?;
            let state = admit_plan(
                &client,
                &owner_session_id,
                &cli.distribution_id,
                cli.distribution_generation,
                &data_dir.join(ENSEMBLE_RESERVATION_FILE),
            )?;
            write_state(&state_path, &state)?;
            remove_reservation(&data_dir.join(ENSEMBLE_RESERVATION_FILE))?;
            println!("development_assembly=admitted");
            Ok(())
        }
        Command::StartEnsemble => {
            let state = read_state(&state_path)?;
            if state.distribution_id != cli.distribution_id
                || state.distribution_generation != cli.distribution_generation
            {
                return Err("development assembly state does not match the release".to_owned());
            }
            let client = client(&data_dir)?;
            let signer = FileOwnerSigner::open(&data_dir)?;
            let owner_session_id = client.open_owner_session(&signer)?;
            start_ensemble(&client, &owner_session_id, &state)?;
            Ok(())
        }
    }
}

fn start_ensemble(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    state: &DevelopmentAssemblyStateV1,
) -> Result<(), String> {
    if state.modules.len() != MODULE_PLAN.len() {
        return Err("development assembly module state is incomplete".to_owned());
    }
    for (plan, module) in MODULE_PLAN.iter().zip(&state.modules) {
        if module.runtime_artifact_id != plan.runtime_artifact_id
            || module.storage_capability_id != plan.storage_capability_id
        {
            return Err("development assembly module state does not match the plan".to_owned());
        }
        match plan.runtime_kind {
            ModuleRuntimeKindV1::Domain => {
                client.start_reserved_domain_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                )?;
            }
            ModuleRuntimeKindV1::Engine => {
                client.start_reserved_engine_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                )?;
            }
            ModuleRuntimeKindV1::Integration => {
                let started = client.start_reserved_integration_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                    plan.configuration_instance_id.ok_or_else(|| {
                        "development integration configuration is absent".to_owned()
                    })?,
                    plan.request_host_bridge,
                )?;
                println!(
                    "{}_runtime={}",
                    plan.runtime_artifact_id, started.launch_state
                );
                continue;
            }
        }
        println!("{}_runtime=accepted", plan.runtime_artifact_id);
    }
    Ok(())
}

fn validate_cli(cli: &Cli) -> Result<(), String> {
    if !cli.data_dir.is_absolute()
        || cli.distribution_id.is_empty()
        || cli.distribution_id.len() > 128
        || !cli.distribution_id.is_ascii()
        || cli.distribution_generation == 0
    {
        return Err("development assembly arguments are invalid".to_owned());
    }
    Ok(())
}

fn client(data_dir: &Path) -> Result<OwnerControlClientV1, String> {
    Ok(OwnerControlClientV1::new(&runtime_directory(data_dir)?))
}

fn runtime_directory(data_dir: &Path) -> Result<PathBuf, String> {
    let directories = directories::ProjectDirs::from("dev", "Hermes", "Hermes Hub")
        .ok_or_else(|| "OS-standard local runtime directory is unavailable".to_owned())?;
    let instance_key = Sha256::digest(data_dir.as_os_str().as_encoded_bytes())
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(directories.cache_dir().join("runtime").join(instance_key))
}

fn provision_platform(data_dir: &Path) -> Result<(), String> {
    let credential_directory = data_dir.join("developer-platform-credentials");
    ensure_private_directory(&credential_directory)?;
    let runtime_directory = runtime_directory(data_dir)?;
    ensure_private_directory(&runtime_directory)?;
    let pgbouncer_directory = runtime_directory.join("storage").join("pgbouncer");
    let pgbouncer_auth_directory = pgbouncer_directory.join("auth");
    ensure_private_directory(&pgbouncer_directory)?;
    ensure_private_directory(&pgbouncer_auth_directory)?;
    write_private_if_absent(&pgbouncer_directory.join("databases.ini"), b"[databases]\n")?;

    for name in [
        "postgres-admin-password",
        "pgbouncer-admin-password",
        "nats-event-hub-password",
    ] {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes)
            .map_err(|_| "development platform credentials are unavailable".to_owned())?;
        write_private_if_absent(&credential_directory.join(name), hex(&bytes).as_bytes())?;
    }

    let seed_path = credential_directory.join("nats-account-signer-seed");
    let public_path = credential_directory.join("nats-account-public-key");
    match (
        std::fs::symlink_metadata(&seed_path),
        std::fs::symlink_metadata(&public_path),
    ) {
        (Err(seed_error), Err(public_error))
            if seed_error.kind() == std::io::ErrorKind::NotFound
                && public_error.kind() == std::io::ErrorKind::NotFound =>
        {
            let signer = nats_jwt::KeyPair::new_account();
            let seed = signer
                .seed()
                .map_err(|_| "development NATS signer is unavailable".to_owned())?;
            write_private_if_absent(&seed_path, seed.as_bytes())?;
            write_private_if_absent(&public_path, signer.public_key().as_bytes())?;
        }
        (Ok(_), Ok(_)) => {
            let seed = read_private_string(&seed_path)?;
            let expected_public = nats_jwt::KeyPair::from_seed(&seed)
                .map_err(|_| "development NATS signer is invalid".to_owned())?
                .public_key();
            if read_private_string(&public_path)? != expected_public {
                return Err("development NATS signer files do not match".to_owned());
            }
        }
        _ => return Err("development NATS signer state is partial".to_owned()),
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_dir()
                && metadata.permissions().mode() & 0o077 == 0 =>
        {
            Ok(())
        }
        Ok(_) => Err("development platform directory is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir_all(path)
            .and_then(|()| std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)))
            .map_err(|_| "development platform directory is unavailable".to_owned()),
        Err(_) => Err("development platform directory is unavailable".to_owned()),
    }
}

fn write_private_if_absent(path: &Path, bytes: &[u8]) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_file()
                && metadata.permissions().mode() & 0o077 == 0
                && metadata.len() > 0 =>
        {
            Ok(())
        }
        Ok(_) => Err("development platform file is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|_| "development platform file is unavailable".to_owned())?;
            file.write_all(bytes)
                .and_then(|()| file.sync_all())
                .map_err(|_| "development platform file is unavailable".to_owned())
        }
        Err(_) => Err("development platform file is unavailable".to_owned()),
    }
}

fn read_private_string(path: &Path) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "development platform file is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() == 0
        || metadata.len() > 4_096
    {
        return Err("development platform file is invalid".to_owned());
    }
    std::fs::read_to_string(path)
        .map(|value| value.trim().to_owned())
        .map_err(|_| "development platform file is unavailable".to_owned())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn admit_plan(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    distribution_id: &str,
    distribution_generation: u64,
    reservation_path: &Path,
) -> Result<DevelopmentAssemblyStateV1, String> {
    if let Some(reservation) = read_reservation_if_present(reservation_path)? {
        validate_reservation_release(&reservation, distribution_id, distribution_generation)?;
        return finish_ensemble_bindings(client, owner_session_id, reservation);
    }

    let mut modules = Vec::with_capacity(MODULE_PLAN.len());
    for module in MODULE_PLAN {
        let proposal = client
            .propose_bundled_managed_artifact(
                owner_session_id,
                module.runtime_artifact_id,
                distribution_id,
                distribution_generation,
                operation_id(module.runtime_artifact_id),
            )
            .map_err(|error| admission_error(module.runtime_artifact_id, "propose", error))?;
        let status = client
            .module_registration_status(&proposal.registration_id)
            .map_err(|error| admission_error(module.runtime_artifact_id, "status", error))?;
        match status.registration_state.as_str() {
            "pending" => {
                client
                    .approve_module_registration(
                        owner_session_id,
                        &proposal.registration_id,
                        proposal.requested_capability_ids.clone(),
                    )
                    .map_err(|error| {
                        admission_error(module.runtime_artifact_id, "approve", error)
                    })?;
            }
            "approved"
                if usize::try_from(status.effective_capability_count).ok()
                    == Some(proposal.requested_capability_ids.len()) => {}
            _ => return Err("development module admission state is invalid".to_owned()),
        }
        client
            .bind_bundled_managed_release(
                owner_session_id,
                &proposal.registration_id,
                module.runtime_artifact_id,
            )
            .map_err(|error| admission_error(module.runtime_artifact_id, "bind_release", error))?;
        let storage = client
            .admit_bundled_storage_artifact(
                owner_session_id,
                module.storage_artifact_id,
                distribution_id,
                distribution_generation,
            )
            .map_err(|error| admission_error(module.runtime_artifact_id, "admit_storage", error))?;
        let storage_capability_id = exact_requested_capability(
            proposal.requested_capability_ids.iter().map(String::as_str),
            module.storage_capability_id,
        )
        .map_err(|error| admission_error(module.runtime_artifact_id, "select_storage", error))?;
        let reservation = client
            .reserve_bundled_managed_runtime(owner_session_id, &proposal.registration_id)
            .map_err(|error| {
                admission_error(module.runtime_artifact_id, "reserve_runtime", error)
            })?;
        modules.push(ModuleReservationV1 {
            runtime_artifact_id: module.runtime_artifact_id.to_owned(),
            registration_id: proposal.registration_id,
            storage_capability_id,
            runtime_instance_id: reservation.runtime_instance_id,
            runtime_generation: reservation.runtime_generation,
            storage_bundle_revision: storage.storage_bundle_revision,
            storage_bundle_digest: storage.storage_bundle_digest.try_into().map_err(|_| {
                admission_error(
                    module.runtime_artifact_id,
                    "admit_storage",
                    "Storage bundle digest is invalid".to_owned(),
                )
            })?,
        });
    }
    let reservation = EnsembleReservationV2 {
        distribution_id: distribution_id.to_owned(),
        distribution_generation,
        modules,
    };
    write_reservation(reservation_path, &reservation)?;
    finish_ensemble_bindings(client, owner_session_id, reservation)
}

fn admission_error(artifact_id: &str, phase: &str, error: String) -> String {
    format!("module={artifact_id} phase={phase}: {error}")
}

fn finish_ensemble_bindings(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    reservation: EnsembleReservationV2,
) -> Result<DevelopmentAssemblyStateV1, String> {
    if reservation.modules.len() != MODULE_PLAN.len() {
        return Err("development ensemble reservation is incomplete".to_owned());
    }
    let mut modules = Vec::with_capacity(MODULE_PLAN.len());
    for (plan, module) in MODULE_PLAN.iter().zip(reservation.modules) {
        if module.runtime_artifact_id != plan.runtime_artifact_id
            || module.storage_capability_id != plan.storage_capability_id
        {
            return Err("development ensemble reservation does not match the plan".to_owned());
        }
        client
            .issue_managed_storage_binding(
                owner_session_id,
                &module.registration_id,
                &module.storage_capability_id,
                &module.runtime_instance_id,
                module.runtime_generation,
                1,
                1,
                module.storage_bundle_revision,
                module.storage_bundle_digest.to_vec(),
            )
            .map_err(|error| {
                admission_error(plan.runtime_artifact_id, "issue_storage_binding", error)
            })?;
        modules.push(ModuleAssemblyStateV1 {
            runtime_artifact_id: module.runtime_artifact_id,
            registration_id: module.registration_id,
            storage_capability_id: module.storage_capability_id,
        });
    }
    Ok(DevelopmentAssemblyStateV1 {
        distribution_id: reservation.distribution_id,
        distribution_generation: reservation.distribution_generation,
        modules,
    })
}

fn exact_requested_capability<'a>(
    capabilities: impl Iterator<Item = &'a str>,
    expected_capability_id: &str,
) -> Result<String, String> {
    let values = capabilities
        .filter(|capability| *capability == expected_capability_id)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [capability] => Ok((*capability).to_owned()),
        _ => Err("module must request its exact Storage capability".to_owned()),
    }
}

fn operation_id(artifact_id: &str) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"hermes.local-development-assembly.proposal.v1");
    digest.update([0]);
    digest.update(artifact_id.as_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has a fixed size")
}

struct DevelopmentAssemblyStateV1 {
    distribution_id: String,
    distribution_generation: u64,
    modules: Vec<ModuleAssemblyStateV1>,
}

struct ModuleAssemblyStateV1 {
    runtime_artifact_id: String,
    registration_id: String,
    storage_capability_id: String,
}

struct EnsembleReservationV2 {
    distribution_id: String,
    distribution_generation: u64,
    modules: Vec<ModuleReservationV1>,
}

struct ModuleReservationV1 {
    runtime_artifact_id: String,
    registration_id: String,
    storage_capability_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
}

fn write_reservation(path: &Path, reservation: &EnsembleReservationV2) -> Result<(), String> {
    if reservation.modules.len() != MODULE_PLAN.len() {
        return Err("development ensemble reservation is incomplete".to_owned());
    }
    let mut bytes = format!(
        "version=2\ndistribution_id={}\ndistribution_generation={}\nmodule_count={}\n",
        reservation.distribution_id,
        reservation.distribution_generation,
        reservation.modules.len(),
    );
    for (index, module) in reservation.modules.iter().enumerate() {
        bytes.push_str(&format!(
            "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id={}\nmodule.{index}.storage_capability_id={}\nmodule.{index}.runtime_instance_id={}\nmodule.{index}.runtime_generation={}\nmodule.{index}.storage_bundle_revision={}\nmodule.{index}.storage_bundle_digest={}\n",
            module.runtime_artifact_id,
            module.registration_id,
            module.storage_capability_id,
            module.runtime_instance_id,
            module.runtime_generation,
            module.storage_bundle_revision,
            hex(&module.storage_bundle_digest),
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "development ensemble reservation cannot be staged".to_owned())?;
    file.write_all(bytes.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|_| "development ensemble reservation cannot be staged".to_owned())
}

fn read_reservation_if_present(path: &Path) -> Result<Option<EnsembleReservationV2>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_file()
                && metadata.permissions().mode() & 0o077 == 0
                && metadata.len() <= 16_384 =>
        {
            read_reservation(path).map(Some)
        }
        Ok(_) => Err("development ensemble reservation is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("development ensemble reservation is unavailable".to_owned()),
    }
}

fn read_reservation(path: &Path) -> Result<EnsembleReservationV2, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| "development ensemble reservation is unavailable".to_owned())?;
    let fields = content
        .lines()
        .map(|line| {
            line.split_once('=')
                .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
    if fields.get("version") != Some(&"2")
        || parse_positive_field(&fields, "module_count")? as usize != MODULE_PLAN.len()
        || fields.len() != 4 + MODULE_PLAN.len() * 7
    {
        return Err("development ensemble reservation is invalid".to_owned());
    }
    let modules = MODULE_PLAN
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let field = |name: &str| format!("module.{index}.{name}");
            let runtime_artifact_id =
                reservation_required_field(&fields, &field("runtime_artifact_id"))?;
            if runtime_artifact_id != plan.runtime_artifact_id {
                return Err("development ensemble reservation is invalid".to_owned());
            }
            Ok(ModuleReservationV1 {
                runtime_artifact_id: runtime_artifact_id.to_owned(),
                registration_id: reservation_required_field(&fields, &field("registration_id"))?
                    .to_owned(),
                storage_capability_id: reservation_required_field(
                    &fields,
                    &field("storage_capability_id"),
                )?
                .to_owned(),
                runtime_instance_id: reservation_required_field(
                    &fields,
                    &field("runtime_instance_id"),
                )?
                .to_owned(),
                runtime_generation: parse_positive_field(&fields, &field("runtime_generation"))?,
                storage_bundle_revision: parse_positive_field(
                    &fields,
                    &field("storage_bundle_revision"),
                )?,
                storage_bundle_digest: decode_hex_32(reservation_required_field(
                    &fields,
                    &field("storage_bundle_digest"),
                )?)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(EnsembleReservationV2 {
        distribution_id: reservation_required_field(&fields, "distribution_id")?.to_owned(),
        distribution_generation: parse_positive_field(&fields, "distribution_generation")?,
        modules,
    })
}

fn reservation_required_field<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<&'a str, String> {
    fields
        .get(name)
        .copied()
        .filter(|value| !value.is_empty() && value.len() <= 256 && value.is_ascii())
        .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
}

fn parse_positive_field(
    fields: &std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<u64, String> {
    reservation_required_field(fields, name)?
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
}

fn decode_hex_32(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("development ensemble reservation is invalid".to_owned());
    }
    let mut output = [0_u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "development ensemble reservation is invalid".to_owned())?;
    }
    Ok(output)
}

fn validate_reservation_release(
    reservation: &EnsembleReservationV2,
    distribution_id: &str,
    distribution_generation: u64,
) -> Result<(), String> {
    (reservation.distribution_id == distribution_id
        && reservation.distribution_generation == distribution_generation)
        .then_some(())
        .ok_or_else(|| "development ensemble reservation does not match the release".to_owned())
}

fn remove_reservation(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("development ensemble reservation cannot be removed".to_owned()),
    }
}

fn write_state(path: &Path, state: &DevelopmentAssemblyStateV1) -> Result<(), String> {
    if state.modules.len() != MODULE_PLAN.len() {
        return Err("development assembly state is incomplete".to_owned());
    }
    if let Ok(metadata) = std::fs::symlink_metadata(path)
        && (metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() > 16_384)
    {
        return Err("development assembly state cannot be replaced".to_owned());
    }
    let mut bytes = format!(
        "version=2\ndistribution_id={}\ndistribution_generation={}\nmodule_count={}\n",
        state.distribution_id,
        state.distribution_generation,
        state.modules.len(),
    );
    for (index, module) in state.modules.iter().enumerate() {
        bytes.push_str(&format!(
            "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id={}\nmodule.{index}.storage_capability_id={}\n",
            module.runtime_artifact_id, module.registration_id, module.storage_capability_id,
        ));
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| "development assembly state cannot be staged".to_owned())?;
    let result = file
        .write_all(bytes.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|_| "development assembly state cannot be staged".to_owned())
        .and_then(|()| {
            std::fs::rename(&temporary, path)
                .map_err(|_| "development assembly state cannot be committed".to_owned())
        });
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn read_state_if_present(path: &Path) -> Result<Option<DevelopmentAssemblyStateV1>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => read_state(path).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("development assembly state is unavailable".to_owned()),
    }
}

fn read_state(path: &Path) -> Result<DevelopmentAssemblyStateV1, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "development assembly state is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > 16_384
    {
        return Err("development assembly state is invalid".to_owned());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|_| "development assembly state is unavailable".to_owned())?;
    let fields = content
        .lines()
        .map(|line| {
            line.split_once('=')
                .ok_or_else(|| "development assembly state is invalid".to_owned())
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
    if fields.get("version") != Some(&"2")
        || required_field(&fields, "module_count")?
            .parse::<usize>()
            .ok()
            != Some(MODULE_PLAN.len())
        || fields.len() != 4 + MODULE_PLAN.len() * 3
    {
        return Err("development assembly state is invalid".to_owned());
    }
    let modules = MODULE_PLAN
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let field = |name: &str| format!("module.{index}.{name}");
            let runtime_artifact_id = required_field(&fields, &field("runtime_artifact_id"))?;
            let storage_capability_id = required_field(&fields, &field("storage_capability_id"))?;
            if runtime_artifact_id != plan.runtime_artifact_id
                || storage_capability_id != plan.storage_capability_id
            {
                return Err("development assembly state is invalid".to_owned());
            }
            Ok(ModuleAssemblyStateV1 {
                runtime_artifact_id: runtime_artifact_id.to_owned(),
                registration_id: required_field(&fields, &field("registration_id"))?.to_owned(),
                storage_capability_id: storage_capability_id.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let state = DevelopmentAssemblyStateV1 {
        distribution_id: required_field(&fields, "distribution_id")?.to_owned(),
        distribution_generation: required_field(&fields, "distribution_generation")?
            .parse()
            .map_err(|_| "development assembly state is invalid".to_owned())?,
        modules,
    };
    if state.distribution_generation == 0
        || std::iter::once(state.distribution_id.as_str())
            .chain(state.modules.iter().flat_map(|module| {
                [
                    module.runtime_artifact_id.as_str(),
                    module.registration_id.as_str(),
                    module.storage_capability_id.as_str(),
                ]
            }))
            .any(|value| value.is_empty() || value.len() > 128 || !value.is_ascii())
    {
        return Err("development assembly state is invalid".to_owned());
    }
    Ok(state)
}

fn required_field<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "development assembly state is invalid".to_owned())
}

struct FileOwnerSigner(SigningKey);

impl FileOwnerSigner {
    fn open(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join(DEVICE_KEY_FILE);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "owner device signer is unavailable".to_owned())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() != 32
        {
            return Err("owner device signer is unavailable".to_owned());
        }
        let mut bytes = [0_u8; 32];
        File::open(path)
            .and_then(|mut file| file.read_exact(&mut bytes))
            .map_err(|_| "owner device signer is unavailable".to_owned())?;
        SigningKey::from_bytes((&bytes).into())
            .map(Self)
            .map_err(|_| "owner device signer is unavailable".to_owned())
    }
}

impl OwnerControlProofSignerV1 for FileOwnerSigner {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String> {
        let signature: Signature = self.0.sign(message);
        Ok(signature.to_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_plan_keeps_domains_engines_and_integrations_as_distinct_artifacts() {
        assert_eq!(MODULE_PLAN.len(), 6);
        assert_eq!(
            MODULE_PLAN
                .iter()
                .map(|module| module.runtime_artifact_id)
                .collect::<Vec<_>>(),
            vec![
                COMMUNICATIONS_RUNTIME_ARTIFACT,
                ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
                MAIL_RUNTIME_ARTIFACT,
                TELEGRAM_RUNTIME_ARTIFACT,
                WHATSAPP_RUNTIME_ARTIFACT,
                ZULIP_RUNTIME_ARTIFACT,
            ],
        );
        assert_eq!(
            MODULE_PLAN[1].runtime_artifact_id,
            "attachment_security.runtime.v1",
        );
        assert_eq!(
            MODULE_PLAN[1].storage_artifact_id,
            "attachment_security.storage.v1",
        );
    }

    #[test]
    fn storage_capability_selection_is_exact() {
        assert_eq!(
            exact_requested_capability(
                ["mail.query.v1", "mail.storage.v1"].into_iter(),
                "mail.storage.v1",
            ),
            Ok("mail.storage.v1".to_owned()),
        );
        assert!(
            exact_requested_capability(["mail.query.v1"].into_iter(), "mail.storage.v1").is_err()
        );
        assert!(
            exact_requested_capability(
                ["mail.storage.v1", "mail.storage.v1"].into_iter(),
                "mail.storage.v1",
            )
            .is_err()
        );
    }

    #[test]
    fn proposal_operation_ids_are_stable_and_artifact_scoped() {
        assert_eq!(
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
        );
        assert_ne!(
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
            operation_id(MAIL_RUNTIME_ARTIFACT),
        );
    }
}
