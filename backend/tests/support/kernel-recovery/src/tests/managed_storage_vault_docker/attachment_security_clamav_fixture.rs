//! Bounded loopback ClamAV INSTREAM fixture for managed Engine conformance.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const CLAMAV_INSTREAM_COMMAND: &[u8; 10] = b"zINSTREAM\0";
const MAX_FIXTURE_CHUNK_BYTES: usize = 64 * 1024;
const MAX_FIXTURE_SCAN_BYTES: usize = 64 * 1024 * 1024;
const THREAT_MARKER: &[u8] = b"fixture-threat";
const MALFORMED_MARKER: &[u8] = b"fixture-malformed";
const DISCONNECT_MARKER: &[u8] = b"fixture-disconnect";
const TIMEOUT_MARKER: &[u8] = b"fixture-timeout";
const HELD_CLEAN_MARKER: &[u8] = b"fixture-held-clean";
const CUSTODY_PROBE_MARKER: &[u8] = b"fixture-custody";
const VAULT_OUTAGE_PROBE_MARKER: &[u8] = b"fixture-vault-outage";
const BLOB_OUTAGE_PROBE_MARKER: &[u8] = b"fixture-blob-outage";
const TARGET_REVOKED_PROBE_MARKER: &[u8] = b"fixture-target-revoked";
const FIXTURE_OUTCOME_COUNT: usize = 10;
const TIMEOUT_RESPONSE_DELAY: Duration = Duration::from_millis(1_500);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(super) enum ClamAvFixtureOutcomeV1 {
    Clean = 0,
    Threat = 1,
    Malformed = 2,
    Disconnect = 3,
    Timeout = 4,
    HeldClean = 5,
    CustodyProbe = 6,
    VaultOutageProbe = 7,
    BlobOutageProbe = 8,
    TargetRevokedProbe = 9,
}

impl ClamAvFixtureOutcomeV1 {
    const fn index(self) -> usize {
        self as usize
    }
}

pub(super) struct AttachmentSecurityClamAvFixture {
    port: u16,
    shutdown: Arc<AtomicBool>,
    outcome_counts: Arc<[AtomicUsize; FIXTURE_OUTCOME_COUNT]>,
    held_scan_started: Arc<AtomicBool>,
    release_held_scan: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl AttachmentSecurityClamAvFixture {
    pub(super) fn start() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .expect("bind loopback ClamAV fixture");
        listener
            .set_nonblocking(true)
            .expect("configure loopback ClamAV fixture");
        let port = listener
            .local_addr()
            .expect("ClamAV fixture address")
            .port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let outcome_counts = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let held_scan_started = Arc::new(AtomicBool::new(false));
        let release_held_scan = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_outcome_counts = Arc::clone(&outcome_counts);
        let worker_held_scan_started = Arc::clone(&held_scan_started);
        let worker_release_held_scan = Arc::clone(&release_held_scan);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if worker_shutdown.load(Ordering::Acquire) {
                            break;
                        }
                        let outcome = serve_scan(
                            stream,
                            &worker_held_scan_started,
                            &worker_release_held_scan,
                        );
                        worker_outcome_counts[outcome.index()].fetch_add(1, Ordering::AcqRel);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            port,
            shutdown,
            outcome_counts,
            held_scan_started,
            release_held_scan,
            worker: Some(worker),
        }
    }

    pub(super) const fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn scan_count(&self) -> usize {
        self.outcome_counts
            .iter()
            .map(|count| count.load(Ordering::Acquire))
            .sum()
    }

    pub(super) fn outcome_count(&self, outcome: ClamAvFixtureOutcomeV1) -> usize {
        self.outcome_counts[outcome.index()].load(Ordering::Acquire)
    }

    pub(super) fn wait_until_held_scan_started(&self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.held_scan_started.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "held ClamAV scan did not start");
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub(super) fn release_held_scan(&self) {
        self.release_held_scan.store(true, Ordering::Release);
    }
}

impl Drop for AttachmentSecurityClamAvFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.release_held_scan.store(true, Ordering::Release);
        let _ = TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.port));
        if let Some(worker) = self.worker.take() {
            worker.join().expect("join loopback ClamAV fixture");
        }
    }
}

fn serve_scan(
    mut stream: TcpStream,
    held_scan_started: &AtomicBool,
    release_held_scan: &AtomicBool,
) -> ClamAvFixtureOutcomeV1 {
    stream
        .set_nonblocking(false)
        .and_then(|_| stream.set_read_timeout(Some(Duration::from_secs(5))))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .expect("configure ClamAV fixture connection");
    let mut command = [0_u8; CLAMAV_INSTREAM_COMMAND.len()];
    stream
        .read_exact(&mut command)
        .expect("read ClamAV INSTREAM command");
    assert_eq!(command, *CLAMAV_INSTREAM_COMMAND);
    let mut payload = Vec::new();
    loop {
        let mut length = [0_u8; 4];
        stream
            .read_exact(&mut length)
            .expect("read ClamAV INSTREAM chunk length");
        let length = usize::try_from(u32::from_be_bytes(length)).expect("ClamAV chunk length");
        if length == 0 {
            break;
        }
        assert!(length <= MAX_FIXTURE_CHUNK_BYTES);
        let total = payload
            .len()
            .checked_add(length)
            .expect("ClamAV fixture scan size");
        assert!(total <= MAX_FIXTURE_SCAN_BYTES);
        let mut chunk = vec![0_u8; length];
        stream
            .read_exact(&mut chunk)
            .expect("read ClamAV INSTREAM chunk");
        payload.extend_from_slice(&chunk);
    }
    assert!(!payload.is_empty());
    let outcome = scan_outcome_for_payload(&payload);
    match outcome {
        ClamAvFixtureOutcomeV1::Clean => {
            write_fixture_response(&mut stream, b"stream: OK\0", "clean");
        }
        ClamAvFixtureOutcomeV1::Threat => {
            write_fixture_response(&mut stream, b"stream: Fixture-Signature FOUND\0", "threat");
        }
        ClamAvFixtureOutcomeV1::Malformed => {
            write_fixture_response(&mut stream, b"stream: BROKEN\0", "malformed");
        }
        ClamAvFixtureOutcomeV1::Disconnect => {}
        ClamAvFixtureOutcomeV1::Timeout => {
            thread::sleep(TIMEOUT_RESPONSE_DELAY);
            let _ = stream
                .write_all(b"stream: OK\0")
                .and_then(|_| stream.flush());
        }
        ClamAvFixtureOutcomeV1::HeldClean => {
            held_scan_started.store(true, Ordering::Release);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !release_held_scan.load(Ordering::Acquire) {
                assert!(
                    Instant::now() < deadline,
                    "held ClamAV scan was not released"
                );
                thread::sleep(Duration::from_millis(10));
            }
            write_fixture_response(&mut stream, b"stream: OK\0", "held clean");
        }
        ClamAvFixtureOutcomeV1::CustodyProbe
        | ClamAvFixtureOutcomeV1::VaultOutageProbe
        | ClamAvFixtureOutcomeV1::BlobOutageProbe
        | ClamAvFixtureOutcomeV1::TargetRevokedProbe => {
            write_fixture_response(&mut stream, b"stream: OK\0", "authority probe");
        }
    }
    outcome
}

fn scan_outcome_for_payload(payload: &[u8]) -> ClamAvFixtureOutcomeV1 {
    for (marker, outcome) in [
        (THREAT_MARKER, ClamAvFixtureOutcomeV1::Threat),
        (MALFORMED_MARKER, ClamAvFixtureOutcomeV1::Malformed),
        (DISCONNECT_MARKER, ClamAvFixtureOutcomeV1::Disconnect),
        (TIMEOUT_MARKER, ClamAvFixtureOutcomeV1::Timeout),
        (HELD_CLEAN_MARKER, ClamAvFixtureOutcomeV1::HeldClean),
        (CUSTODY_PROBE_MARKER, ClamAvFixtureOutcomeV1::CustodyProbe),
        (
            VAULT_OUTAGE_PROBE_MARKER,
            ClamAvFixtureOutcomeV1::VaultOutageProbe,
        ),
        (
            BLOB_OUTAGE_PROBE_MARKER,
            ClamAvFixtureOutcomeV1::BlobOutageProbe,
        ),
        (
            TARGET_REVOKED_PROBE_MARKER,
            ClamAvFixtureOutcomeV1::TargetRevokedProbe,
        ),
    ] {
        if payload.windows(marker.len()).any(|window| window == marker) {
            return outcome;
        }
    }
    ClamAvFixtureOutcomeV1::Clean
}

fn write_fixture_response(stream: &mut TcpStream, response: &[u8], label: &str) {
    stream
        .write_all(response)
        .and_then(|_| stream.flush())
        .unwrap_or_else(|error| panic!("write ClamAV {label} response: {error}"));
}

#[cfg(test)]
mod tests {
    use super::{ClamAvFixtureOutcomeV1, scan_outcome_for_payload};

    #[test]
    fn fixture_classifies_each_scanner_outcome_from_bounded_payload() {
        assert_eq!(
            scan_outcome_for_payload(b"ordinary attachment"),
            ClamAvFixtureOutcomeV1::Clean
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-threat marker"),
            ClamAvFixtureOutcomeV1::Threat
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-malformed marker"),
            ClamAvFixtureOutcomeV1::Malformed
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-disconnect marker"),
            ClamAvFixtureOutcomeV1::Disconnect
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-timeout marker"),
            ClamAvFixtureOutcomeV1::Timeout
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-held-clean marker"),
            ClamAvFixtureOutcomeV1::HeldClean
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-custody marker"),
            ClamAvFixtureOutcomeV1::CustodyProbe
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-vault-outage marker"),
            ClamAvFixtureOutcomeV1::VaultOutageProbe
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-blob-outage marker"),
            ClamAvFixtureOutcomeV1::BlobOutageProbe
        );
        assert_eq!(
            scan_outcome_for_payload(b"attachment fixture-target-revoked marker"),
            ClamAvFixtureOutcomeV1::TargetRevokedProbe
        );
    }
}
