//! Development platform probes and pairing simulation commands.

use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use crate::identity::file_signer::FileDeviceSigner;
use crate::pairing::{state as pairing_state, tls as pairing_tls};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) fn probe(postgres: SocketAddr, nats: SocketAddr) -> Result<(), String> {
    probe_endpoint("PostgreSQL", postgres)?;
    probe_endpoint("NATS", nats)?;
    println!("development-platform-runtime: PostgreSQL and NATS TCP endpoints reachable");
    Ok(())
}

pub(crate) fn pairing_create(state_dir: &Path, ttl_seconds: u64) -> Result<(), String> {
    let token = pairing_state::create(state_dir, ttl_seconds)?;
    println!("development_pairing_token={token}");
    println!("development_pairing_state_dir={}", state_dir.display());
    Ok(())
}

pub(crate) fn pairing_consume(state_dir: &Path, token: &str) -> Result<(), String> {
    pairing_state::consume(state_dir, token)?;
    println!("development_pairing_consumed=true");
    Ok(())
}

pub(crate) fn pairing_listen(
    state_dir: &Path,
    listen_address: SocketAddr,
    idle_timeout_seconds: u64,
) -> Result<(), String> {
    if !(1..=900).contains(&idle_timeout_seconds) {
        return Err("--idle-timeout-seconds must be between 1 and 900".to_owned());
    }
    pairing_tls::listen(pairing_tls::ListenerConfig {
        state_dir,
        listen_address,
        idle_timeout: Duration::from_secs(idle_timeout_seconds),
    })
}

pub(crate) fn pairing_proof(
    key_dir: &Path,
    challenge: &str,
    owner_id: &str,
    device_id: &str,
) -> Result<(), String> {
    let challenge = pairing_tls::decode_hex::<32>(challenge, "challenge")?;
    let signer = FileDeviceSigner::open_or_create(key_dir)?;
    let public_key = signer.public_key_sec1();
    let message = pairing_tls::proof_message(&challenge, &public_key, owner_id, device_id)?;
    println!("device_public_key_sec1={}", pairing_tls::hex(&public_key));
    println!(
        "device_signature_raw={}",
        pairing_tls::hex(&signer.sign(&message))
    );
    Ok(())
}

fn probe_endpoint(name: &str, address: SocketAddr) -> Result<(), String> {
    TcpStream::connect_timeout(&address, CONNECT_TIMEOUT)
        .map(|_| ())
        .map_err(|error| format!("{name} at {address} is unavailable: {error}"))
}
