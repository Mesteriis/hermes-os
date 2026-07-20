//! Explicit local operator ceremony for admitting one browser device.

use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use hermes_gateway_protocol::v1::{
    BeginBrowserPairingRequestV1, BeginOwnerControlSessionRequestV1,
    CompleteOwnerControlSessionRequestV1, OwnerControlRequestV1, OwnerControlResponseV1,
    owner_control_request_v1, owner_control_response_v1,
};
use prost::Message;

use super::sessions::{OwnerControlChallenge, client_proof_message};
use super::{read_frame, write_frame};
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::infrastructure::filesystem::{resolve_data_directory, resolve_runtime_directory};

const IPC_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn create_browser_pairing(data_dir_override: Option<PathBuf>) -> Result<(), String> {
    confirm_creation()?;
    let data_dir = resolve_data_directory(data_dir_override)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let session_id = open_owner_session(&runtime_dir, &data_dir)?;
    let pairing_id = begin_browser_pairing(&runtime_dir, &session_id)?;
    println!("browser_pairing_id={pairing_id}");
    println!("browser_pairing_state=approved_pending_registration");
    Ok(())
}

fn confirm_creation() -> Result<(), String> {
    eprint!("Create one-time browser pairing for a new device? [y/N] ");
    io::stderr().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| error.to_string())?;
    matches!(answer.trim(), "y" | "Y" | "yes" | "YES")
        .then_some(())
        .ok_or_else(|| "browser pairing was not confirmed".to_owned())
}

fn connect_owner_socket(runtime_dir: &std::path::Path) -> Result<UnixStream, String> {
    let stream = UnixStream::connect(runtime_dir.join("owner.sock"))
        .map_err(|_| "owner control socket is unavailable".to_owned())?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
        .map_err(|_| "owner control socket is unavailable".to_owned())?;
    Ok(stream)
}

fn open_owner_session(
    runtime_dir: &std::path::Path,
    data_dir: &std::path::Path,
) -> Result<String, String> {
    let mut stream = connect_owner_socket(runtime_dir)?;
    let response = request(
        &mut stream,
        owner_control_request_v1::Operation::BeginOwnerSession(
            BeginOwnerControlSessionRequestV1 {},
        ),
    )?;
    let challenge = match response.result {
        Some(owner_control_response_v1::Result::BeginOwnerSession(value)) => value,
        _ => return Err("owner control session is unavailable".to_owned()),
    };
    let challenge = owner_challenge(challenge)?;
    let signer = FileDeviceSigner::open_for_instance(data_dir)?;
    let signature = signer.sign(&client_proof_message(&challenge)?);
    let mut stream = connect_owner_socket(runtime_dir)?;
    let response = request(
        &mut stream,
        owner_control_request_v1::Operation::CompleteOwnerSession(
            CompleteOwnerControlSessionRequestV1 {
                challenge_id: challenge.challenge_id().to_owned(),
                signature_raw: signature.to_vec(),
            },
        ),
    )?;
    match response.result {
        Some(owner_control_response_v1::Result::CompleteOwnerSession(value))
            if !value.owner_session_id.is_empty() =>
        {
            Ok(value.owner_session_id)
        }
        _ => Err("owner control session is unavailable".to_owned()),
    }
}

fn owner_challenge(
    response: hermes_gateway_protocol::v1::BeginOwnerControlSessionResponseV1,
) -> Result<OwnerControlChallenge, String> {
    let bytes: [u8; 32] = response
        .challenge_bytes
        .try_into()
        .map_err(|_| "owner control challenge is invalid".to_owned())?;
    OwnerControlChallenge::new_for_client(
        response.challenge_id,
        bytes,
        response.kernel_instance_id,
        response.owner_id,
        response.device_id,
        response.control_store_generation,
        response.expires_at_unix_millis,
    )
}

fn begin_browser_pairing(
    runtime_dir: &std::path::Path,
    session_id: &str,
) -> Result<String, String> {
    let mut stream = connect_owner_socket(runtime_dir)?;
    let response = request(
        &mut stream,
        owner_control_request_v1::Operation::BeginBrowserPairing(BeginBrowserPairingRequestV1 {
            owner_session_id: session_id.to_owned(),
        }),
    )?;
    match response.result {
        Some(owner_control_response_v1::Result::BeginBrowserPairing(value))
            if value.pairing_id.len() == 64 =>
        {
            Ok(value.pairing_id)
        }
        _ => Err("browser pairing is unavailable".to_owned()),
    }
}

fn request(
    stream: &mut UnixStream,
    operation: owner_control_request_v1::Operation,
) -> Result<OwnerControlResponseV1, String> {
    let request = OwnerControlRequestV1 {
        operation: Some(operation),
    };
    write_frame(stream, &request.encode_to_vec())?;
    let response = OwnerControlResponseV1::decode(read_frame(stream)?.as_slice())
        .map_err(|_| "owner control response is invalid".to_owned())?;
    response
        .error_code
        .is_empty()
        .then_some(response)
        .ok_or_else(|| "owner control operation was denied".to_owned())
}
