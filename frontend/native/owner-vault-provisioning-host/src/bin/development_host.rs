use std::fs;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use hermes_owner_vault_provisioning_host::{
    AuthorizedProvisioningV1, CommittedProvisioningReceiptV1, OwnerVaultProvisioningHostV1,
};
use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use zeroize::Zeroizing;

const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:9445";
const EXACT_BROWSER_ORIGIN: &str = "http://127.0.0.1:5173";
const PROOF_HEADER: &str = "x-hermes-development-host-proof";
const JSON_CONTENT_TYPE: &str = "application/json";
const MAX_REQUEST_BYTES: usize = 256 * 1024;
const START_PATH: &str = "/__hermes/owner-vault-host/v1/start";
const SEAL_PATH: &str = "/__hermes/owner-vault-host/v1/seal";
const OPEN_RECEIPT_PATH: &str = "/__hermes/owner-vault-host/v1/open-receipt";
const CANCEL_PATH: &str = "/__hermes/owner-vault-host/v1/cancel";

fn main() -> Result<(), String> {
    let configuration = DevelopmentHostConfigurationV1::from_args(std::env::args().skip(1))?;
    let proof = load_private_proof(&configuration.proof_file)?;
    let server = Server::http(configuration.listen_address)
        .map_err(|_| "development host listen failed".to_owned())?;
    let host = Arc::new(OwnerVaultProvisioningHostV1::default());
    println!(
        "Hermes owner Vault development host is ready at {}",
        configuration.listen_address
    );
    for request in server.incoming_requests() {
        serve_request(request, &host, &proof);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DevelopmentHostConfigurationV1 {
    listen_address: SocketAddr,
    proof_file: PathBuf,
}

impl DevelopmentHostConfigurationV1 {
    fn from_args(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut listen_address = SocketAddr::from_str(DEFAULT_LISTEN_ADDRESS)
            .map_err(|_| "development host listen address is invalid".to_owned())?;
        let mut proof_file = None;
        let mut args = args.peekable();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--listen-address" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "development host listen address is missing".to_owned())?;
                    listen_address = SocketAddr::from_str(&value)
                        .map_err(|_| "development host listen address is invalid".to_owned())?;
                }
                "--proof-file" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "development host proof file is missing".to_owned())?;
                    proof_file = Some(PathBuf::from(value));
                }
                _ => return Err("development host argument is unsupported".to_owned()),
            }
        }
        if !matches!(listen_address.ip(), IpAddr::V4(address) if address.is_loopback()) {
            return Err("development host must bind an IPv4 loopback address".to_owned());
        }
        let proof_file =
            proof_file.ok_or_else(|| "development host proof file is required".to_owned())?;
        if !proof_file.is_absolute() {
            return Err("development host proof file must be absolute".to_owned());
        }
        Ok(Self {
            listen_address,
            proof_file,
        })
    }
}

fn load_private_proof(path: &Path) -> Result<Zeroizing<String>, String> {
    let link_metadata = fs::symlink_metadata(path)
        .map_err(|_| "development host proof file is unavailable".to_owned())?;
    if !link_metadata.file_type().is_file() || link_metadata.file_type().is_symlink() {
        return Err("development host proof file is invalid".to_owned());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if link_metadata.permissions().mode() & 0o077 != 0 {
            return Err("development host proof file permissions are invalid".to_owned());
        }
    }
    let proof = Zeroizing::new(
        fs::read_to_string(path)
            .map_err(|_| "development host proof file is unreadable".to_owned())?,
    );
    if proof.len() != 64 || !proof.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("development host proof is invalid".to_owned());
    }
    Ok(proof)
}

fn serve_request(mut request: Request, host: &OwnerVaultProvisioningHostV1, proof: &str) {
    let response = dispatch(&mut request, host, proof).unwrap_or_else(|error| error.response());
    let _ = request.respond(response);
}

fn dispatch(
    request: &mut Request,
    host: &OwnerVaultProvisioningHostV1,
    proof: &str,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, DevelopmentHostErrorV1> {
    authorize_request(request, proof)?;
    match request.url() {
        START_PATH => {
            require_empty_body(request)?;
            let started = host
                .start()
                .map_err(|_| DevelopmentHostErrorV1::Unavailable)?;
            json_response(
                StatusCode(200),
                &StartedProvisioningHostSessionResponseV1 {
                    host_session_id: started.host_session_id,
                    response_recipient_hpke_public_key_x25519: started
                        .response_recipient_hpke_public_key_x25519
                        .to_vec(),
                },
            )
        }
        SEAL_PATH => {
            let request: SealProvisioningCommandRequestV1 = json_request(request)?;
            let sealed = host
                .seal(
                    &request.host_session_id,
                    request.authorized.try_into()?,
                    exact_array(request.operation_id)?,
                    request.action,
                    request.secret_class,
                    request.secret_payload,
                )
                .map_err(|_| DevelopmentHostErrorV1::Rejected)?;
            json_response(
                StatusCode(200),
                &SealedProvisioningCommandResponseV1 {
                    operation_digest_sha256: sealed.operation_digest_sha256.to_vec(),
                    hpke_encapped_key: sealed.hpke_encapped_key,
                    ciphertext: sealed.ciphertext,
                    hpke_authentication_tag: sealed.hpke_authentication_tag,
                },
            )
        }
        OPEN_RECEIPT_PATH => {
            let request: OpenProvisioningReceiptRequestV1 = json_request(request)?;
            let receipt = host
                .open_receipt(&request.host_session_id, request.committed.try_into()?)
                .map_err(|_| DevelopmentHostErrorV1::Rejected)?;
            json_response(
                StatusCode(200),
                &SanitizedProvisioningReceiptResponseV1 {
                    operation_id: receipt.operation_id.to_vec(),
                    action: receipt.action,
                    secret_revision: receipt.secret_revision.to_string(),
                    state: receipt.state,
                },
            )
        }
        CANCEL_PATH => {
            let request: CancelProvisioningHostSessionRequestV1 = json_request(request)?;
            host.cancel(&request.host_session_id)
                .map_err(|_| DevelopmentHostErrorV1::Rejected)?;
            json_response(StatusCode(200), &EmptyResponseV1 {})
        }
        _ => Err(DevelopmentHostErrorV1::NotFound),
    }
}

fn authorize_request(request: &Request, proof: &str) -> Result<(), DevelopmentHostErrorV1> {
    authorize_request_metadata(
        request.method(),
        request.url(),
        header_value(request, "origin"),
        header_value(request, PROOF_HEADER),
        header_value(request, "content-length"),
        proof,
    )
}

fn authorize_request_metadata(
    method: &Method,
    url: &str,
    origin: Option<&str>,
    supplied_proof: Option<&str>,
    content_length: Option<&str>,
    proof: &str,
) -> Result<(), DevelopmentHostErrorV1> {
    if method != &Method::Post || url.contains('?') {
        return Err(DevelopmentHostErrorV1::NotFound);
    }
    let origin = origin.ok_or(DevelopmentHostErrorV1::Denied)?;
    let supplied_proof = supplied_proof.ok_or(DevelopmentHostErrorV1::Denied)?;
    if origin != EXACT_BROWSER_ORIGIN || supplied_proof.as_bytes() != proof.as_bytes() {
        return Err(DevelopmentHostErrorV1::Denied);
    }
    if let Some(length) = content_length {
        let length = length
            .parse::<usize>()
            .map_err(|_| DevelopmentHostErrorV1::InvalidRequest)?;
        if length > MAX_REQUEST_BYTES {
            return Err(DevelopmentHostErrorV1::InvalidRequest);
        }
    }
    Ok(())
}

fn header_value<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|header| header.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn require_empty_body(request: &mut Request) -> Result<(), DevelopmentHostErrorV1> {
    let body = bounded_body(request)?;
    if body.is_empty() || body.as_slice() == b"{}" {
        Ok(())
    } else {
        Err(DevelopmentHostErrorV1::InvalidRequest)
    }
}

fn json_request<T>(request: &mut Request) -> Result<T, DevelopmentHostErrorV1>
where
    T: for<'de> Deserialize<'de>,
{
    if header_value(request, "content-type") != Some(JSON_CONTENT_TYPE) {
        return Err(DevelopmentHostErrorV1::InvalidRequest);
    }
    let body = bounded_body(request)?;
    serde_json::from_slice(&body).map_err(|_| DevelopmentHostErrorV1::InvalidRequest)
}

fn bounded_body(request: &mut Request) -> Result<Zeroizing<Vec<u8>>, DevelopmentHostErrorV1> {
    let mut body = Zeroizing::new(Vec::new());
    request
        .as_reader()
        .take((MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| DevelopmentHostErrorV1::InvalidRequest)?;
    if body.len() > MAX_REQUEST_BYTES {
        return Err(DevelopmentHostErrorV1::InvalidRequest);
    }
    Ok(body)
}

fn json_response<T>(
    status: StatusCode,
    value: &T,
) -> Result<Response<std::io::Cursor<Vec<u8>>>, DevelopmentHostErrorV1>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(value).map_err(|_| DevelopmentHostErrorV1::Unavailable)?;
    Ok(Response::from_data(bytes)
        .with_status_code(status)
        .with_header(json_header())
        .with_header(no_store_header()))
}

fn json_header() -> Header {
    Header::from_bytes("content-type", JSON_CONTENT_TYPE)
        .expect("static JSON content type header must be valid")
}

fn no_store_header() -> Header {
    Header::from_bytes("cache-control", "no-store")
        .expect("static cache-control header must be valid")
}

fn exact_array<const N: usize>(value: Vec<u8>) -> Result<[u8; N], DevelopmentHostErrorV1> {
    value
        .try_into()
        .map_err(|_| DevelopmentHostErrorV1::InvalidRequest)
}

#[derive(Debug)]
enum DevelopmentHostErrorV1 {
    InvalidRequest,
    Denied,
    NotFound,
    Rejected,
    Unavailable,
}

impl DevelopmentHostErrorV1 {
    fn response(self) -> Response<std::io::Cursor<Vec<u8>>> {
        let (status, code) = match self {
            Self::InvalidRequest => (400, "invalid_request"),
            Self::Denied => (403, "denied"),
            Self::NotFound => (404, "not_found"),
            Self::Rejected => (409, "rejected"),
            Self::Unavailable => (503, "unavailable"),
        };
        let body = serde_json::to_vec(&ErrorResponseV1 { code })
            .expect("static development host error response must serialize");
        Response::from_data(body)
            .with_status_code(StatusCode(status))
            .with_header(json_header())
            .with_header(no_store_header())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseV1 {
    code: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartedProvisioningHostSessionResponseV1 {
    host_session_id: String,
    response_recipient_hpke_public_key_x25519: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SealProvisioningCommandRequestV1 {
    host_session_id: String,
    operation_id: Vec<u8>,
    action: i32,
    secret_class: i32,
    secret_payload: Vec<u8>,
    authorized: AuthorizedProvisioningRequestV1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthorizedProvisioningRequestV1 {
    vault_runtime_generation: String,
    vault_hpke_public_key_x25519: Vec<u8>,
    audience_registration_id: String,
    audience_runtime_instance_id: String,
    audience_runtime_generation: String,
    audience_grant_epoch: String,
    lease_request_id: Vec<u8>,
    lease_operation_digest_sha256: Vec<u8>,
    command_request_id: Vec<u8>,
    lease_response_hpke_encapped_key: Vec<u8>,
    lease_response_ciphertext: Vec<u8>,
    lease_response_hpke_authentication_tag: Vec<u8>,
}

impl TryFrom<AuthorizedProvisioningRequestV1> for AuthorizedProvisioningV1 {
    type Error = DevelopmentHostErrorV1;

    fn try_from(value: AuthorizedProvisioningRequestV1) -> Result<Self, Self::Error> {
        Ok(Self {
            vault_runtime_generation: unsigned(&value.vault_runtime_generation)?,
            vault_hpke_public_key_x25519: exact_array(value.vault_hpke_public_key_x25519)?,
            audience_registration_id: value.audience_registration_id,
            audience_runtime_instance_id: value.audience_runtime_instance_id,
            audience_runtime_generation: unsigned(&value.audience_runtime_generation)?,
            audience_grant_epoch: unsigned(&value.audience_grant_epoch)?,
            lease_request_id: exact_array(value.lease_request_id)?,
            lease_operation_digest_sha256: exact_array(value.lease_operation_digest_sha256)?,
            command_request_id: exact_array(value.command_request_id)?,
            lease_response_hpke_encapped_key: value.lease_response_hpke_encapped_key,
            lease_response_ciphertext: value.lease_response_ciphertext,
            lease_response_hpke_authentication_tag: value.lease_response_hpke_authentication_tag,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SealedProvisioningCommandResponseV1 {
    operation_digest_sha256: Vec<u8>,
    hpke_encapped_key: Vec<u8>,
    ciphertext: Vec<u8>,
    hpke_authentication_tag: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OpenProvisioningReceiptRequestV1 {
    host_session_id: String,
    committed: CommittedProvisioningReceiptRequestV1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommittedProvisioningReceiptRequestV1 {
    vault_runtime_generation: String,
    command_request_id: Vec<u8>,
    operation_digest_sha256: Vec<u8>,
    receipt_hpke_encapped_key: Vec<u8>,
    receipt_ciphertext: Vec<u8>,
    receipt_hpke_authentication_tag: Vec<u8>,
}

impl TryFrom<CommittedProvisioningReceiptRequestV1> for CommittedProvisioningReceiptV1 {
    type Error = DevelopmentHostErrorV1;

    fn try_from(value: CommittedProvisioningReceiptRequestV1) -> Result<Self, Self::Error> {
        Ok(Self {
            vault_runtime_generation: unsigned(&value.vault_runtime_generation)?,
            command_request_id: exact_array(value.command_request_id)?,
            operation_digest_sha256: exact_array(value.operation_digest_sha256)?,
            receipt_hpke_encapped_key: value.receipt_hpke_encapped_key,
            receipt_ciphertext: value.receipt_ciphertext,
            receipt_hpke_authentication_tag: value.receipt_hpke_authentication_tag,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SanitizedProvisioningReceiptResponseV1 {
    operation_id: Vec<u8>,
    action: i32,
    secret_revision: String,
    state: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CancelProvisioningHostSessionRequestV1 {
    host_session_id: String,
}

#[derive(Serialize)]
struct EmptyResponseV1 {}

fn unsigned(value: &str) -> Result<u64, DevelopmentHostErrorV1> {
    value
        .parse::<u64>()
        .map_err(|_| DevelopmentHostErrorV1::InvalidRequest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rejects_non_loopback_and_relative_proof_paths() {
        assert_eq!(
            DevelopmentHostConfigurationV1::from_args(
                ["--proof-file", "/tmp/proof"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .expect("valid configuration")
            .listen_address,
            SocketAddr::from_str(DEFAULT_LISTEN_ADDRESS).expect("default address")
        );
        assert!(
            DevelopmentHostConfigurationV1::from_args(
                [
                    "--listen-address",
                    "0.0.0.0:9445",
                    "--proof-file",
                    "/tmp/proof"
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            DevelopmentHostConfigurationV1::from_args(
                ["--proof-file", "proof"].into_iter().map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn wire_conversion_requires_exact_fences_and_unsigned_generations() {
        let authorized = AuthorizedProvisioningRequestV1 {
            vault_runtime_generation: "1".to_owned(),
            vault_hpke_public_key_x25519: vec![1; 31],
            audience_registration_id: "telegram-registration".to_owned(),
            audience_runtime_instance_id: "telegram-runtime".to_owned(),
            audience_runtime_generation: "2".to_owned(),
            audience_grant_epoch: "3".to_owned(),
            lease_request_id: vec![4; 16],
            lease_operation_digest_sha256: vec![5; 32],
            command_request_id: vec![6; 16],
            lease_response_hpke_encapped_key: vec![7; 32],
            lease_response_ciphertext: vec![8],
            lease_response_hpke_authentication_tag: vec![9; 16],
        };
        assert!(AuthorizedProvisioningV1::try_from(authorized).is_err());
        assert!(unsigned("-1").is_err());
        assert!(unsigned("18446744073709551616").is_err());
    }

    #[test]
    fn request_admission_requires_exact_loopback_origin_proof_and_bound() {
        let proof = "a".repeat(64);
        assert!(
            authorize_request_metadata(
                &Method::Post,
                START_PATH,
                Some(EXACT_BROWSER_ORIGIN),
                Some(&proof),
                Some("2"),
                &proof,
            )
            .is_ok()
        );

        for denied in [
            authorize_request_metadata(
                &Method::Post,
                START_PATH,
                Some("http://localhost:5173"),
                Some(&proof),
                Some("2"),
                &proof,
            ),
            authorize_request_metadata(
                &Method::Post,
                START_PATH,
                Some(EXACT_BROWSER_ORIGIN),
                Some("wrong-proof"),
                Some("2"),
                &proof,
            ),
        ] {
            assert!(matches!(denied, Err(DevelopmentHostErrorV1::Denied)));
        }

        assert!(matches!(
            authorize_request_metadata(
                &Method::Get,
                START_PATH,
                Some(EXACT_BROWSER_ORIGIN),
                Some(&proof),
                None,
                &proof,
            ),
            Err(DevelopmentHostErrorV1::NotFound),
        ));
        assert!(matches!(
            authorize_request_metadata(
                &Method::Post,
                START_PATH,
                Some(EXACT_BROWSER_ORIGIN),
                Some(&proof),
                Some("262145"),
                &proof,
            ),
            Err(DevelopmentHostErrorV1::InvalidRequest),
        ));
    }
}
