//! Loopback TLS Gmail API provider used by the managed Mail delivery contour.

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rcgen::generate_simple_self_signed;
use rustls::{
    ServerConfig, ServerConnection, StreamOwned,
    pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
};

const MAX_HTTP_LINE_BYTES: usize = 8 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub(super) struct GmailSentRequestV1 {
    pub(super) path: String,
    pub(super) authorization: String,
    pub(super) raw: String,
    pub(super) thread_id: String,
}

pub(super) struct MailGmailFixture {
    port: u16,
    ca_certificate_pem: String,
    accepted_mutations: Arc<AtomicUsize>,
    last_request: Arc<Mutex<Option<GmailSentRequestV1>>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MailGmailFixture {
    pub(super) fn start() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
            .expect("generate Gmail fixture certificate");
        let ca_certificate_pem = certified.cert.pem();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let server = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("configure Gmail fixture TLS"),
        );
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind Gmail fixture");
        listener
            .set_nonblocking(true)
            .expect("configure Gmail fixture listener");
        let port = listener.local_addr().expect("Gmail fixture address").port();
        let accepted_mutations = Arc::new(AtomicUsize::new(0));
        let last_request = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_mutations = Arc::clone(&accepted_mutations);
        let worker_request = Arc::clone(&last_request);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("configure blocking Gmail fixture connection");
                        stream
                            .set_read_timeout(Some(Duration::from_secs(10)))
                            .expect("configure Gmail fixture read timeout");
                        stream
                            .set_write_timeout(Some(Duration::from_secs(10)))
                            .expect("configure Gmail fixture write timeout");
                        let connection =
                            ServerConnection::new(Arc::clone(&server)).expect("Gmail TLS session");
                        let mut stream = StreamOwned::new(connection, stream);
                        serve_connection(&mut stream, &worker_mutations, &worker_request);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("Gmail fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            ca_certificate_pem,
            accepted_mutations,
            last_request,
            shutdown,
            worker: Some(worker),
        }
    }

    pub(super) fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn ca_certificate_pem(&self) -> &str {
        &self.ca_certificate_pem
    }

    pub(super) fn accepted_mutations(&self) -> usize {
        self.accepted_mutations.load(Ordering::SeqCst)
    }

    pub(super) fn last_request(&self) -> GmailSentRequestV1 {
        self.last_request
            .lock()
            .expect("lock Gmail fixture request")
            .clone()
            .expect("Gmail fixture request")
    }
}

impl Drop for MailGmailFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let outcome = worker.join();
            if !std::thread::panicking() {
                outcome.expect("join Gmail fixture");
            }
        }
    }
}

fn serve_connection(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    accepted_mutations: &AtomicUsize,
    last_request: &Mutex<Option<GmailSentRequestV1>>,
) {
    let request_line = read_line(stream);
    let request_line = std::str::from_utf8(&request_line).expect("Gmail request line");
    let mut parts = request_line.split_whitespace();
    assert_eq!(parts.next(), Some("POST"));
    let path = parts.next().expect("Gmail request path").to_owned();
    assert_eq!(parts.next(), Some("HTTP/1.1"));
    assert_eq!(parts.next(), None);

    let mut headers = BTreeMap::new();
    loop {
        let line = read_line(stream);
        if line == b"\r\n" {
            break;
        }
        let line = std::str::from_utf8(&line).expect("Gmail request header");
        let (name, value) = line
            .trim_end()
            .split_once(':')
            .expect("Gmail request header shape");
        assert!(
            headers
                .insert(name.to_ascii_lowercase(), value.trim().to_owned())
                .is_none(),
            "duplicate Gmail request header"
        );
    }
    let content_length = headers
        .get("content-length")
        .expect("Gmail request content length")
        .parse::<usize>()
        .expect("Gmail request content length value");
    assert!(content_length <= MAX_REQUEST_BODY_BYTES);
    let mut body = vec![0_u8; content_length];
    stream
        .read_exact(&mut body)
        .expect("read Gmail request body");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("decode Gmail request body");
    let raw = body
        .get("raw")
        .and_then(serde_json::Value::as_str)
        .expect("Gmail raw message")
        .to_owned();
    let thread_id = body
        .get("threadId")
        .and_then(serde_json::Value::as_str)
        .expect("Gmail thread ID")
        .to_owned();
    let authorization = headers
        .get("authorization")
        .expect("Gmail authorization")
        .to_owned();
    *last_request.lock().expect("lock Gmail fixture request") = Some(GmailSentRequestV1 {
        path,
        authorization,
        raw,
        thread_id,
    });
    accepted_mutations.fetch_add(1, Ordering::SeqCst);

    let body = br#"{"id":"gmail-sent-1","threadId":"gmail-thread-1","labelIds":["SENT"]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .and_then(|_| stream.write_all(body))
        .and_then(|_| stream.flush())
        .expect("write Gmail response");
}

fn read_line(stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>) -> Vec<u8> {
    let mut line = Vec::new();
    while line.len() <= MAX_HTTP_LINE_BYTES {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("read Gmail request");
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            return line;
        }
    }
    panic!("Gmail fixture line exceeded its bound");
}
