use super::common::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::identity::browser_gateway::ControlStoreBrowserAuthority;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use hermes_gateway_protocol::{
    v1::{ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame},
    validation::validate_client_realtime_frame,
};
use hermes_gateway_runtime::{
    BrowserAuthenticationRouter, BrowserPairingRouter, BrowserRealtimeSubscriptionSource,
    ClientRealtimeSubscriptionV1, GatewayApplicationRouter,
};
use hermes_gateway_session::{
    BrowserCredentialMaterialV1, BrowserGatewaySessionService, BrowserPairingManager,
    BrowserSameOriginSessionV1, BrowserSession, BrowserWebauthnVerifier, OwnerPairingApprovalV1,
};
use hermes_gateway_session_contract::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    BrowserEnrollmentAuthority, BrowserEnrollmentV1, GatewayIdentityFenceV1,
};
use hermes_kernel_control_store::BrowserDeviceEnrollmentV1;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use sha2::{Digest, Sha256};
use webauthn_rs_core::proto::{COSEAlgorithm, COSEEC2Key, COSEKey, COSEKeyType, ECDSACurve};

struct ClosedReplaySource;

fn browser_authority(store: Arc<SqliteControlStore>) -> ControlStoreBrowserAuthority {
    ControlStoreBrowserAuthority::new(
        store,
        ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false))),
    )
}

impl BrowserRealtimeSubscriptionSource for ClosedReplaySource {
    fn subscribe(
        &self,
        session: &BrowserSession,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String> {
        if session.owner_id() != "owner-1" || after_cursor.is_some() {
            return Err("unexpected browser session".to_owned());
        }
        let (sender, receiver) = tokio::sync::broadcast::channel(1);
        drop(sender);
        ClientRealtimeSubscriptionV1::new(
            vec![ClientRealtimeFrameV1 {
                frame: Some(Frame::Event(ClientRealtimeEventV1 {
                    event_id: vec![7; 16],
                    cursor: "cursor-1".to_owned(),
                    contract_name: "hermes.client.status".to_owned(),
                    contract_version: 1,
                    event_kind: "status_changed".to_owned(),
                    occurred_at_unix_millis: 1,
                    causation_id: String::new(),
                    correlation_id: String::new(),
                    trace_id: String::new(),
                    payload: b"client-safe".to_vec(),
                })),
            }],
            receiver,
        )
    }
}

#[path = "browser_gateway_session/credential.rs"]
mod credential;

#[test]
fn browser_authentication_http_flow_issues_a_cookie_once_and_persists_the_counter() {
    let fixture = authentication_http_fixture();
    let (authentication_id, challenge, browser_key_challenge) =
        begin_browser_authentication(&fixture);
    assert_invalid_browser_local_key_proof_is_rejected(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    let cookie = finish_browser_authentication(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    assert_realtime_session_flow(&fixture, &cookie);
    assert_authentication_replay_is_rejected(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

struct AuthenticationHttpFixture {
    root: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    router: GatewayApplicationRouter<ControlStoreBrowserAuthority, ClosedReplaySource>,
    runtime: tokio::runtime::Runtime,
}

fn authentication_http_fixture() -> AuthenticationHttpFixture {
    let root = unique_target_root("hermes-browser-gateway-http-authentication");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    store
        .admit_browser_device(
            &BrowserDeviceEnrollmentV1::new(
                "owner-1",
                "browser-1",
                vec![1],
                valid_browser_cose_key(),
                valid_browser_local_key(),
                "hub.local",
                0,
                false,
                false,
            )
            .expect("valid browser enrollment"),
            1,
        )
        .expect("admit browser");

    let verifier =
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("verifier");
    let service = BrowserGatewaySessionService::new(
        browser_authority(Arc::clone(&store)),
        verifier,
        "https://hub.local",
    )
    .expect("browser session service");
    let service = std::sync::Arc::new(service);
    let router = GatewayApplicationRouter::new(true, service, ClosedReplaySource);
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let health = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Full::new(Bytes::new()))
                .expect("health request"),
        ),
    );
    assert_eq!(health.status(), StatusCode::OK);
    AuthenticationHttpFixture {
        root,
        store,
        router,
        runtime,
    }
}

fn begin_browser_authentication(fixture: &AuthenticationHttpFixture) -> (String, String, String) {
    let begin = fixture.runtime.block_on(
        fixture
            .router
            .route(begin_authentication_request("https://hub.local")),
    );
    assert_eq!(begin.status(), StatusCode::OK);
    let begin_body = fixture
        .runtime
        .block_on(begin.into_body().collect())
        .expect("begin response body")
        .to_bytes();
    let ceremony: serde_json::Value = serde_json::from_slice(&begin_body).expect("begin JSON");
    let authentication_id = ceremony["authentication_id"]
        .as_str()
        .expect("authentication ID");
    let challenge = ceremony["public_key"]["challenge"]
        .as_str()
        .expect("WebAuthn challenge");
    let browser_key_challenge = ceremony["browser_key_challenge"]
        .as_str()
        .expect("browser key challenge");
    (
        authentication_id.to_owned(),
        challenge.to_owned(),
        browser_key_challenge.to_owned(),
    )
}

fn finish_browser_authentication(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) -> String {
    let request = finish_authentication_request(
        authentication_id,
        &signed_browser_assertion(challenge, 1),
        browser_key_challenge,
    );
    let response = fixture.runtime.block_on(fixture.router.route(request));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .expect("cache control"),
        "no-store"
    );
    let cookie = response
        .headers()
        .get("set-cookie")
        .expect("secure browser cookie")
        .to_str()
        .expect("cookie header");
    assert!(cookie.starts_with("__Host-hermes-session="));
    assert!(cookie.contains("Secure; HttpOnly; SameSite=Strict"));
    assert_eq!(
        fixture
            .store
            .browser_device_identity("browser-1")
            .expect("read browser identity")
            .expect("browser identity exists")
            .enrollment()
            .sign_count(),
        1
    );
    cookie
        .split(';')
        .next()
        .expect("session cookie pair")
        .to_owned()
}

fn assert_realtime_session_flow(fixture: &AuthenticationHttpFixture, session_cookie: &str) {
    let realtime = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", session_cookie)
                .body(Full::new(Bytes::new()))
                .expect("realtime request"),
        ),
    );
    assert_eq!(realtime.status(), StatusCode::OK);
    assert_eq!(
        realtime
            .headers()
            .get("content-type")
            .expect("SSE content type"),
        "text/event-stream"
    );
    let realtime_body = fixture
        .runtime
        .block_on(realtime.into_body().collect())
        .expect("SSE response body")
        .to_bytes();
    let realtime_body = std::str::from_utf8(&realtime_body).expect("SSE UTF-8");
    assert!(realtime_body.starts_with("id: cursor-1\nevent: hermes.realtime.v1\ndata: "));
    assert!(realtime_body.ends_with("\n\n"));
    assert!(!realtime_body.contains("client-safe"));
    let invalid_replay = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events?cursor=cursor-1")
                .header("cookie", session_cookie)
                .body(Full::new(Bytes::new()))
                .expect("realtime request"),
        ),
    );
    assert_eq!(invalid_replay.status(), StatusCode::NOT_FOUND);
}

fn assert_authentication_replay_is_rejected(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) {
    let replay = fixture
        .runtime
        .block_on(fixture.router.route(finish_authentication_request(
            authentication_id,
            &signed_browser_assertion(challenge, 2),
            browser_key_challenge,
        )));
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
}

fn assert_invalid_browser_local_key_proof_is_rejected(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) {
    let mut invalid_signature = signed_browser_key_proof(browser_key_challenge);
    let replacement = if invalid_signature.starts_with('A') {
        "B"
    } else {
        "A"
    };
    invalid_signature.replace_range(0..1, replacement);
    let response = fixture.runtime.block_on(fixture.router.route(
        finish_authentication_request_with_browser_key_signature(
            authentication_id,
            &signed_browser_assertion(challenge, 1),
            &invalid_signature,
        ),
    ));
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[path = "browser_gateway_session/realtime_contract.rs"]
mod realtime_contract;

#[path = "browser_gateway_session/connect_status.rs"]
mod connect_status;

fn begin_authentication_request(origin: &str) -> Request<Full<Bytes>> {
    Request::builder()
        .method("POST")
        .uri("/browser/v1/authentication/begin")
        .header("origin", origin)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from_static(b"{\"credential_id\":\"AQ\"}")))
        .expect("browser authentication request")
}

fn finish_authentication_request(
    authentication_id: &str,
    assertion: &str,
    browser_key_challenge: &str,
) -> Request<Full<Bytes>> {
    finish_authentication_request_with_browser_key_signature(
        authentication_id,
        assertion,
        &signed_browser_key_proof(browser_key_challenge),
    )
}

fn finish_authentication_request_with_browser_key_signature(
    authentication_id: &str,
    assertion: &str,
    browser_key_signature: &str,
) -> Request<Full<Bytes>> {
    Request::builder()
        .method("POST")
        .uri(format!(
            "/browser/v1/authentication/{authentication_id}/finish"
        ))
        .header("origin", "https://hub.local")
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(format!(
            r#"{{"credential":{assertion},"browser_key_signature":"{}"}}"#,
            browser_key_signature,
        ))))
        .expect("browser authentication finish request")
}

fn signed_browser_assertion(challenge: &str, sign_count: u32) -> String {
    let client_data = format!(
        r#"{{"type":"webauthn.get","challenge":"{challenge}","origin":"https://hub.local","crossOrigin":false}}"#
    );
    let client_data_hash = Sha256::digest(client_data.as_bytes());
    let mut authenticator_data = Sha256::digest(b"hub.local").to_vec();
    authenticator_data.push(0x05);
    authenticator_data.extend_from_slice(&sign_count.to_be_bytes());
    let mut signed_data = authenticator_data.clone();
    signed_data.extend_from_slice(&client_data_hash);
    let signature: Signature = browser_signing_key().sign(&signed_data);
    format!(
        r#"{{"id":"AQ","rawId":"AQ","type":"public-key","response":{{"authenticatorData":"{}","clientDataJSON":"{}","signature":"{}","userHandle":null}},"clientExtensionResults":{{}}}}"#,
        base64_url_encode(&authenticator_data),
        base64_url_encode(client_data.as_bytes()),
        base64_url_encode(signature.to_der().as_bytes()),
    )
}

fn base64_url_encode(value: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::with_capacity((value.len() * 4).div_ceil(3));
    for chunk in value.chunks(3) {
        let first = chunk[0];
        encoded.push(char::from(ALPHABET[usize::from(first >> 2)]));
        encoded.push(char::from(
            ALPHABET
                [usize::from(((first & 0x03) << 4) | (chunk.get(1).copied().unwrap_or(0) >> 4))],
        ));
        if let Some(second) = chunk.get(1) {
            encoded.push(char::from(
                ALPHABET[usize::from(
                    ((second & 0x0f) << 2) | (chunk.get(2).copied().unwrap_or(0) >> 6),
                )],
            ));
        }
        if let Some(third) = chunk.get(2) {
            encoded.push(char::from(ALPHABET[usize::from(third & 0x3f)]));
        }
    }
    encoded
}

fn browser_signing_key() -> SigningKey {
    SigningKey::from_bytes((&[7_u8; 32]).into()).expect("test signing key")
}

fn valid_browser_local_key() -> Vec<u8> {
    browser_signing_key()
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .to_vec()
}

fn signed_browser_key_proof(challenge: &str) -> String {
    let raw = decode_base64_url(challenge);
    let signature: Signature = browser_signing_key().sign(&raw);
    base64_url_encode(&signature.to_bytes())
}

fn decode_base64_url(value: &str) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = Vec::with_capacity(value.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in value.bytes() {
        let value = ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)
            .expect("base64url character") as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
        }
    }
    output
}

fn valid_browser_cose_key() -> Vec<u8> {
    let signing_key = browser_signing_key();
    let point = signing_key.verifying_key().to_sec1_point(false);
    let key = COSEKey {
        type_: COSEAlgorithm::ES256,
        key: COSEKeyType::EC_EC2(COSEEC2Key {
            curve: ECDSACurve::SECP256R1,
            x: point.x().expect("P-256 x coordinate").to_vec().into(),
            y: point.y().expect("P-256 y coordinate").to_vec().into(),
        }),
    };
    serde_cbor_2::to_vec(&key).expect("serialize COSE key")
}

#[path = "browser_gateway_session/authority.rs"]
mod authority;

#[path = "browser_gateway_session/enrollment.rs"]
mod enrollment;

#[path = "browser_gateway_session/pairing.rs"]
mod pairing;

#[path = "browser_gateway_session/webauthn.rs"]
mod webauthn;

#[test]
fn browser_pairing_http_exposes_only_an_owner_approved_ceremony_and_keeps_it_after_origin_reject() {
    let fixture = pairing_http_fixture();
    assert_unavailable_pairing_is_rejected(&fixture);
    assert_pairing_options_are_origin_bound(&fixture);
    assert_wrong_origin_keeps_pairing_available(&fixture);
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

struct PairingHttpFixture {
    root: std::path::PathBuf,
    pairing_id: String,
    router: GatewayApplicationRouter<ControlStoreBrowserAuthority, ClosedReplaySource>,
    runtime: tokio::runtime::Runtime,
}

fn pairing_http_fixture() -> PairingHttpFixture {
    let root = unique_target_root("hermes-browser-pairing-http");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let authority = browser_authority(Arc::clone(&store));
    let pairings = Arc::new(std::sync::Mutex::new(BrowserPairingManager::default()));
    let ceremony = pairings
        .lock()
        .expect("pairing lock")
        .begin_webauthn(
            &authority,
            &BrowserWebauthnVerifier::new("hub.local", "https://hub.local")
                .expect("pairing verifier"),
            OwnerPairingApprovalV1::new("owner-1", "desktop-1").expect("pairing approval"),
            1_000,
        )
        .expect("owner-approved pairing");
    let pairing_id = ceremony.pairing().pairing_id().to_owned();
    let session = BrowserGatewaySessionService::new(
        authority.clone(),
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("session verifier"),
        "https://hub.local",
    )
    .expect("browser session service");
    let router = GatewayApplicationRouter::new(true, Arc::new(session), ClosedReplaySource)
        .with_browser_pairing(BrowserPairingRouter::new(
            Arc::clone(&pairings),
            authority,
            BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("route verifier"),
            "https://hub.local",
        ));
    PairingHttpFixture {
        root,
        pairing_id,
        router,
        runtime: tokio::runtime::Runtime::new().expect("test runtime"),
    }
}

fn assert_unavailable_pairing_is_rejected(fixture: &PairingHttpFixture) {
    let unavailable = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    "f".repeat(64)
                ))
                .body(Full::new(Bytes::new()))
                .expect("unavailable pairing request"),
        ),
    );
    assert_eq!(unavailable.status(), StatusCode::UNAUTHORIZED);
}

fn assert_pairing_options_are_origin_bound(fixture: &PairingHttpFixture) {
    let options = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    fixture.pairing_id
                ))
                .body(Full::new(Bytes::new()))
                .expect("pairing options request"),
        ),
    );
    assert_eq!(options.status(), StatusCode::OK);
    assert_eq!(
        options.headers().get("cache-control"),
        Some(&"no-store".parse().unwrap())
    );
    let options_body = fixture
        .runtime
        .block_on(options.into_body().collect())
        .expect("options body")
        .to_bytes();
    let options: serde_json::Value = serde_json::from_slice(&options_body).expect("options JSON");
    assert_eq!(options["public_key"]["publicKey"]["rp"]["id"], "hub.local");
}

fn assert_wrong_origin_keeps_pairing_available(fixture: &PairingHttpFixture) {
    let rejected = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration/finish",
                    fixture.pairing_id
                ))
                .header("content-type", "application/json")
                .header("origin", "https://other.local")
                .body(Full::new(Bytes::from_static(b"{}")))
                .expect("wrong-origin registration request"),
        ),
    );
    assert_eq!(rejected.status(), StatusCode::FORBIDDEN);
    let still_available = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    fixture.pairing_id
                ))
                .body(Full::new(Bytes::new()))
                .expect("pairing options retry"),
        ),
    );
    assert_eq!(still_available.status(), StatusCode::OK);
}

#[path = "browser_gateway_session/session_policy.rs"]
mod session_policy;

#[path = "browser_gateway_session/owner_epoch.rs"]
mod owner_epoch;
