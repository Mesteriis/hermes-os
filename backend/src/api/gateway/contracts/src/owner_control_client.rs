//! Typed client for the owner-private Kernel Unix control socket.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use prost::Message;

use crate::owner_control_proof::owner_control_proof_message_v1;
use crate::v1::{
    BeginBrowserPairingRequestV1, BeginOwnerControlSessionRequestV1,
    BeginOwnerControlSessionResponseV1, CompleteOwnerControlSessionRequestV1,
    OwnerControlRequestV1, OwnerControlResponseV1, owner_control_request_v1,
    owner_control_response_v1,
};

const IPC_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_FRAME_BYTES: usize = 64 * 1024;

pub trait OwnerControlProofSignerV1 {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String>;
}

pub struct OwnerControlClientV1 {
    socket_path: PathBuf,
}

pub struct OwnerControlChallengeV1 {
    challenge_id: String,
    challenge_bytes: [u8; 32],
    kernel_instance_id: String,
    owner_id: String,
    device_id: String,
    control_store_generation: u64,
}

impl OwnerControlClientV1 {
    #[must_use]
    pub fn new(runtime_dir: &Path) -> Self {
        Self {
            socket_path: runtime_dir.join("owner.sock"),
        }
    }

    pub fn open_owner_session(
        &self,
        signer: &impl OwnerControlProofSignerV1,
    ) -> Result<String, String> {
        let response = self.request(owner_control_request_v1::Operation::BeginOwnerSession(
            BeginOwnerControlSessionRequestV1 {},
        ))?;
        let challenge = match response.result {
            Some(owner_control_response_v1::Result::BeginOwnerSession(value)) => {
                OwnerControlChallengeV1::from_response(value)?
            }
            _ => return Err("owner control session is unavailable".to_owned()),
        };
        let signature = signer.sign_owner_control_proof(&challenge.proof_message()?)?;
        let response = self.request(owner_control_request_v1::Operation::CompleteOwnerSession(
            CompleteOwnerControlSessionRequestV1 {
                challenge_id: challenge.challenge_id,
                signature_raw: signature.to_vec(),
            },
        ))?;
        match response.result {
            Some(owner_control_response_v1::Result::CompleteOwnerSession(value))
                if !value.owner_session_id.is_empty() =>
            {
                Ok(value.owner_session_id)
            }
            _ => Err("owner control session is unavailable".to_owned()),
        }
    }

    pub fn begin_browser_pairing(&self, owner_session_id: &str) -> Result<String, String> {
        let response = self.request(owner_control_request_v1::Operation::BeginBrowserPairing(
            BeginBrowserPairingRequestV1 {
                owner_session_id: owner_session_id.to_owned(),
            },
        ))?;
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
        &self,
        operation: owner_control_request_v1::Operation,
    ) -> Result<OwnerControlResponseV1, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|_| "owner control socket is unavailable".to_owned())?;
        stream
            .set_read_timeout(Some(IPC_TIMEOUT))
            .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
            .map_err(|_| "owner control socket is unavailable".to_owned())?;
        let request = OwnerControlRequestV1 {
            operation: Some(operation),
        };
        write_frame(&mut stream, &request.encode_to_vec())?;
        let response = OwnerControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
            .map_err(|_| "owner control response is invalid".to_owned())?;
        response
            .error_code
            .is_empty()
            .then_some(response)
            .ok_or_else(|| "owner control operation was denied".to_owned())
    }
}

impl OwnerControlChallengeV1 {
    fn from_response(response: BeginOwnerControlSessionResponseV1) -> Result<Self, String> {
        let challenge_bytes: [u8; 32] = response
            .challenge_bytes
            .try_into()
            .map_err(|_| "owner control challenge is invalid".to_owned())?;
        (valid_identifier(&response.kernel_instance_id)
            && valid_identifier(&response.owner_id)
            && valid_identifier(&response.device_id)
            && response.challenge_id.len() == 64
            && response.challenge_id.bytes().all(|byte| byte.is_ascii_hexdigit())
            && response.control_store_generation > 0
            && response.expires_at_unix_millis > 0)
            .then_some(Self {
                challenge_id: response.challenge_id,
                challenge_bytes,
                kernel_instance_id: response.kernel_instance_id,
                owner_id: response.owner_id,
                device_id: response.device_id,
                control_store_generation: response.control_store_generation,
            })
            .ok_or_else(|| "owner control challenge is invalid".to_owned())
    }

    fn proof_message(&self) -> Result<Vec<u8>, String> {
        owner_control_proof_message_v1(
            &self.kernel_instance_id,
            &self.owner_id,
            &self.device_id,
            self.control_store_generation,
            &self.challenge_bytes,
        )
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "owner control frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("owner control frame is too large".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("owner control frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| "owner control request is too large".to_owned())?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
