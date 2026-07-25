//! Bounded loopback ClamAV INSTREAM fixture for managed Engine conformance.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CLAMAV_INSTREAM_COMMAND: &[u8; 10] = b"zINSTREAM\0";
const MAX_FIXTURE_CHUNK_BYTES: usize = 64 * 1024;
const MAX_FIXTURE_SCAN_BYTES: usize = 64 * 1024 * 1024;

pub(super) struct AttachmentSecurityClamAvFixture {
    port: u16,
    shutdown: Arc<AtomicBool>,
    scan_count: Arc<AtomicUsize>,
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
        let scan_count = Arc::new(AtomicUsize::new(0));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_scan_count = Arc::clone(&scan_count);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        serve_clean_scan(stream);
                        worker_scan_count.fetch_add(1, Ordering::AcqRel);
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
            scan_count,
            worker: Some(worker),
        }
    }

    pub(super) const fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn scan_count(&self) -> usize {
        self.scan_count.load(Ordering::Acquire)
    }
}

impl Drop for AttachmentSecurityClamAvFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        let _ = TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.port));
        if let Some(worker) = self.worker.take() {
            worker.join().expect("join loopback ClamAV fixture");
        }
    }
}

fn serve_clean_scan(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .expect("configure ClamAV fixture connection");
    let mut command = [0_u8; CLAMAV_INSTREAM_COMMAND.len()];
    stream
        .read_exact(&mut command)
        .expect("read ClamAV INSTREAM command");
    assert_eq!(command, *CLAMAV_INSTREAM_COMMAND);
    let mut total = 0_usize;
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
        total = total.checked_add(length).expect("ClamAV fixture scan size");
        assert!(total <= MAX_FIXTURE_SCAN_BYTES);
        let mut chunk = vec![0_u8; length];
        stream
            .read_exact(&mut chunk)
            .expect("read ClamAV INSTREAM chunk");
    }
    assert!(total > 0);
    stream
        .write_all(b"stream: OK\0")
        .and_then(|_| stream.flush())
        .expect("write ClamAV clean response");
}
