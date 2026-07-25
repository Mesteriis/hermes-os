//! Loopback HTTPS Zulip API fixture with an explicit conformance-only CA.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
};

use super::*;

pub(super) struct ZulipHttpsFixture {
    realm_url: String,
    ca_certificate_path: PathBuf,
    accepted_connections: Arc<AtomicU64>,
    shutdown: Arc<AtomicBool>,
    server: Option<std::thread::JoinHandle<()>>,
}

impl ZulipHttpsFixture {
    pub(super) fn start(root: &Path) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (ca_certificate, server_certificate, server_key) = certificate_chain();
        let certificate = server_certificate.der().clone();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(server_key.serialize_der()),
        );
        let config = Arc::new(
            rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("Zulip fixture TLS configuration"),
        );
        let ca_certificate_path = root.join("zulip-conformance-ca.pem");
        std::fs::write(&ca_certificate_path, ca_certificate.pem())
            .expect("write Zulip conformance CA");
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind Zulip HTTPS fixture");
        listener
            .set_nonblocking(true)
            .expect("configure Zulip HTTPS fixture");
        let port = listener.local_addr().expect("Zulip fixture address").port();
        let accepted_connections = Arc::new(AtomicU64::new(0));
        let server_connections = Arc::clone(&accepted_connections);
        let shutdown = Arc::new(AtomicBool::new(false));
        let server_shutdown = Arc::clone(&shutdown);
        let server = std::thread::spawn(move || {
            serve(listener, config, &server_shutdown, server_connections);
        });
        Self {
            realm_url: format!("https://localhost:{port}"),
            ca_certificate_path,
            accepted_connections,
            shutdown,
            server: Some(server),
        }
    }

    pub(super) fn realm_url(&self) -> &str {
        &self.realm_url
    }

    pub(super) fn ca_certificate_path(&self) -> &Path {
        &self.ca_certificate_path
    }

    pub(super) fn accepted_connections(&self) -> u64 {
        self.accepted_connections.load(Ordering::Relaxed)
    }
}

impl Drop for ZulipHttpsFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(server) = self.server.take() {
            server.join().expect("join Zulip HTTPS fixture");
        }
    }
}

fn certificate_chain() -> (Certificate, Certificate, KeyPair) {
    let mut ca_parameters =
        CertificateParams::new(Vec::<String>::new()).expect("empty CA subject alternative names");
    ca_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_parameters
        .key_usages
        .push(KeyUsagePurpose::DigitalSignature);
    ca_parameters.key_usages.push(KeyUsagePurpose::KeyCertSign);
    let ca_key = KeyPair::generate().expect("Zulip fixture CA key");
    let ca_certificate = ca_parameters
        .self_signed(&ca_key)
        .expect("self-signed Zulip fixture CA");

    let mut server_parameters =
        CertificateParams::new(vec!["localhost".to_owned()]).expect("localhost certificate");
    server_parameters
        .key_usages
        .push(KeyUsagePurpose::DigitalSignature);
    server_parameters
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    let server_key = KeyPair::generate().expect("Zulip fixture server key");
    let server_certificate = server_parameters
        .signed_by(&server_key, &ca_certificate, &ca_key)
        .expect("CA-signed Zulip fixture certificate");
    (ca_certificate, server_certificate, server_key)
}

fn serve(
    listener: TcpListener,
    config: Arc<rustls::ServerConfig>,
    shutdown: &AtomicBool,
    accepted_connections: Arc<AtomicU64>,
) {
    let mut connections = Vec::new();
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((tcp, _)) => {
                let connection_config = Arc::clone(&config);
                let connection_count = Arc::clone(&accepted_connections);
                connections.push(std::thread::spawn(move || {
                    match serve_connection(tcp, connection_config) {
                        Ok(()) => {
                            connection_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(error) if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() => {
                            fixture_diagnostic(&format!(
                                "developer_zulip_fixture_connection_error={error}"
                            ));
                        }
                        Err(_) => {}
                    }
                }));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("Zulip HTTPS fixture accept failed: {error}"),
        }
    }
    for connection in connections {
        connection.join().expect("join Zulip HTTPS connection");
    }
}

fn fixture_diagnostic(message: &str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        let diagnostic = format!("{message}\n");
        let mut stderr = std::io::stderr().lock();
        let _ = std::io::Write::write_all(&mut stderr, diagnostic.as_bytes());
    }
}

fn serve_connection(
    tcp: TcpStream,
    config: Arc<rustls::ServerConfig>,
) -> Result<(), std::io::Error> {
    tcp.set_nonblocking(false)?;
    tcp.set_read_timeout(Some(Duration::from_secs(2)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(2)))?;
    let connection = rustls::ServerConnection::new(config).map_err(std::io::Error::other)?;
    let mut stream = rustls::StreamOwned::new(connection, tcp);
    let request = read_request(&mut stream)?;
    let request_line = request
        .split(|byte| *byte == b'\n')
        .next()
        .and_then(|line| std::str::from_utf8(line).ok())
        .unwrap_or_default();
    let (status, body) = if request_line.starts_with("POST /api/v1/register ") {
        (
            "200 OK",
            r#"{"result":"success","msg":"","queue_id":"managed-zulip-queue","last_event_id":0}"#,
        )
    } else if request_line.starts_with("GET /api/v1/events?") {
        ("200 OK", r#"{"result":"success","msg":"","events":[]}"#)
    } else if request_line.starts_with("POST /api/v1/messages ") {
        ("200 OK", r#"{"result":"success","msg":"","id":4242}"#)
    } else {
        (
            "404 Not Found",
            r#"{"result":"error","msg":"unknown route"}"#,
        )
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn read_request(
    stream: &mut rustls::StreamOwned<rustls::ServerConnection, TcpStream>,
) -> Result<Vec<u8>, std::io::Error> {
    const MAX_REQUEST_BYTES: usize = 65_536;
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4_096];
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(std::io::Error::other("Zulip fixture request is too large"));
        }
        if request_is_complete(&request) {
            break;
        }
    }
    (!request.is_empty())
        .then_some(request)
        .ok_or_else(|| std::io::Error::other("Zulip fixture request is empty"))
}

fn request_is_complete(request: &[u8]) -> bool {
    let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = match std::str::from_utf8(&request[..header_end]) {
        Ok(headers) => headers,
        Err(_) => return false,
    };
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    request.len() >= header_end + 4 + content_length
}
