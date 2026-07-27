//! Loopback-only IMAP protocol fixture for the feature-gated managed Mail conformance binary.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FIXTURE_USERNAME: &str = "owner@example.test";
const FIXTURE_PASSWORD: &str = "managed-mail-imap-password";
const FIXTURE_UID: u32 = 42;
const FIXTURE_MESSAGE: &[u8] = concat!(
    "From: source@example.test\r\n",
    "To: owner@example.test\r\n",
    "Subject: managed attachment evidence\r\n",
    "Content-Type: multipart/mixed; boundary=hermes-fixture\r\n",
    "\r\n",
    "--hermes-fixture\r\n",
    "Content-Type: text/plain; charset=utf-8\r\n",
    "\r\n",
    "managed Mail body\r\n",
    "--hermes-fixture\r\n",
    "Content-Type: application/pdf; name=evidence.pdf\r\n",
    "Content-Disposition: attachment; filename=evidence.pdf\r\n",
    "Content-Transfer-Encoding: base64\r\n",
    "\r\n",
    "Y2xlYW4tcm9vbS1hdHRhY2htZW50\r\n",
    "--hermes-fixture--\r\n",
)
.as_bytes();

pub(super) struct MailImapFixture {
    port: u16,
    shutdown: Arc<AtomicBool>,
    accepted_connections: Arc<AtomicUsize>,
    message_flag_mutations: Arc<AtomicUsize>,
    worker: Option<JoinHandle<()>>,
}

impl MailImapFixture {
    pub(super) fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback IMAP fixture");
        listener
            .set_nonblocking(true)
            .expect("configure loopback IMAP fixture");
        let port = listener
            .local_addr()
            .expect("read loopback IMAP fixture address")
            .port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let accepted_connections = Arc::new(AtomicUsize::new(0));
        let message_flag_mutations = Arc::new(AtomicUsize::new(0));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_connections = Arc::clone(&accepted_connections);
        let worker_message_flag_mutations = Arc::clone(&message_flag_mutations);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if worker_shutdown.load(Ordering::Acquire) {
                            break;
                        }
                        worker_connections.fetch_add(1, Ordering::AcqRel);
                        serve_connection(stream, Arc::clone(&worker_message_flag_mutations));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => panic!("accept loopback IMAP fixture connection"),
                }
            }
        });
        Self {
            port,
            shutdown,
            accepted_connections,
            message_flag_mutations,
            worker: Some(worker),
        }
    }

    pub(super) const fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn accepted_connections(&self) -> usize {
        self.accepted_connections.load(Ordering::Acquire)
    }

    pub(super) fn message_flag_mutations(&self) -> usize {
        self.message_flag_mutations.load(Ordering::Acquire)
    }
}

impl Drop for MailImapFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(worker) = self.worker.take() {
            let result = worker.join();
            if !std::thread::panicking() {
                result.expect("join loopback IMAP fixture");
            }
        }
    }
}

fn serve_connection(mut stream: TcpStream, message_flag_mutations: Arc<AtomicUsize>) {
    stream
        .set_nonblocking(false)
        .and_then(|_| stream.set_read_timeout(Some(Duration::from_secs(15))))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(15))))
        .expect("configure loopback IMAP fixture connection");
    stream
        .write_all(b"* OK Hermes IMAP4rev1 fixture ready\r\n")
        .expect("write IMAP fixture greeting");
    let reader_stream = stream.try_clone().expect("clone fixture read stream");
    let mut lines = BufReader::new(reader_stream).lines();
    while let Some(command) = lines.next().transpose().expect("read IMAP fixture command") {
        let tag = command.split_whitespace().next().expect("IMAP command tag");
        let upper = command.to_ascii_uppercase();
        if upper.contains(" LOGIN ") {
            if !command.contains(FIXTURE_USERNAME) || !command.contains(FIXTURE_PASSWORD) {
                write_tagged(&mut stream, tag, "NO authentication failed");
                continue;
            }
            write_tagged(&mut stream, tag, "OK LOGIN completed");
        } else if upper.contains(" CAPABILITY") {
            write!(
                stream,
                "* CAPABILITY IMAP4rev1\r\n{tag} OK CAPABILITY completed\r\n"
            )
            .expect("write IMAP CAPABILITY response");
        } else if upper.contains(" EXAMINE ") {
            write!(
                stream,
                "* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n\
                 {tag} OK [READ-ONLY] EXAMINE completed\r\n"
            )
            .expect("write IMAP EXAMINE response");
        } else if upper.contains(" SELECT ") {
            write!(
                stream,
                "* FLAGS (\\Seen \\Flagged)\r\n* 1 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n\
                 {tag} OK [READ-WRITE] SELECT completed\r\n"
            )
            .expect("write IMAP SELECT response");
        } else if upper.contains(" UID SEARCH ") {
            write!(
                stream,
                "* SEARCH {FIXTURE_UID}\r\n{tag} OK UID SEARCH completed\r\n"
            )
            .expect("write IMAP UID SEARCH response");
        } else if upper.contains(" UID FETCH ") {
            write!(
                stream,
                "* 1 FETCH (UID {FIXTURE_UID} RFC822.SIZE {} INTERNALDATE \
                 \"24-Jul-2026 12:00:00 +0000\" BODY[] {{{}}}\r\n",
                FIXTURE_MESSAGE.len(),
                FIXTURE_MESSAGE.len(),
            )
            .and_then(|_| stream.write_all(FIXTURE_MESSAGE))
            .and_then(|_| write!(stream, ")\r\n{tag} OK UID FETCH completed\r\n"))
            .expect("write IMAP UID FETCH response");
        } else if upper.contains(" UID STORE ") {
            assert!(
                upper.contains(&format!("UID STORE {FIXTURE_UID}"))
                    && upper.contains("FLAGS.SILENT")
                    && (upper.contains("\\SEEN") || upper.contains("\\FLAGGED")),
                "Mail flag mutation must use exact bounded UID and supported provider flag"
            );
            message_flag_mutations.fetch_add(1, Ordering::AcqRel);
            write_tagged(&mut stream, tag, "OK UID STORE completed");
        } else if upper.contains(" LOGOUT") {
            write!(
                stream,
                "* BYE fixture complete\r\n{tag} OK LOGOUT completed\r\n"
            )
            .expect("write IMAP LOGOUT response");
            break;
        } else {
            write_tagged(&mut stream, tag, "BAD unsupported fixture command");
        }
        stream.flush().expect("flush IMAP fixture response");
    }
}

fn write_tagged(stream: &mut TcpStream, tag: &str, response: &str) {
    writeln!(stream, "{tag} {response}\r").expect("write tagged IMAP fixture response");
    stream.flush().expect("flush tagged IMAP fixture response");
}
