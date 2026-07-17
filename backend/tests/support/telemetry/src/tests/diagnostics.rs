use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::collector::control;
use crate::collector::storage::{TelemetryRetentionV1, TelemetrySegmentStore};
use crate::fixtures::directory::unique_directory;
use hermes_telemetry_protocol::{
    TelemetryPriorityV1, TelemetrySignalKindV1, TelemetrySignalV1, TelemetrySourceV1,
};

#[test]
fn collector_returns_only_aggregate_diagnostics_over_its_inherited_channel() {
    let directory = unique_directory("diagnostics");
    let store = TelemetrySegmentStore::open(
        directory.clone(),
        TelemetryRetentionV1::new(1024, 2048, 60).expect("retention"),
    )
    .expect("open store");
    store.append(&signal()).expect("append signal");
    let (mut kernel, collector) = UnixStream::pair().expect("control pair");
    let worker = std::thread::spawn(move || control::serve_diagnostics(collector, store));

    write_frame(&mut kernel, b"hermes.telemetry.diagnostics.v1");
    let response = String::from_utf8(read_frame(&mut kernel)).expect("diagnostics text");
    assert!(response.starts_with("hermes.telemetry.diagnostics.v1|1|"));
    assert!(!response.contains("runtime-42"));
    drop(kernel);
    worker
        .join()
        .expect("collector worker")
        .expect("serve diagnostics");
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

fn signal() -> TelemetrySignalV1 {
    TelemetrySignalV1::new(
        1,
        TelemetrySourceV1::new("runtime-42".to_owned(), "module.lifecycle".to_owned())
            .expect("source"),
        TelemetrySignalKindV1::Lifecycle,
        TelemetryPriorityV1::Info,
        "runtime.lifecycle.transition".to_owned(),
        None,
        None,
        0,
    )
    .expect("signal")
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    stream
        .write_all(&[bytes.len() as u8])
        .expect("frame length");
    stream.write_all(bytes).expect("frame bytes");
    stream.flush().expect("flush frame");
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = [0_u8; 1];
    stream.read_exact(&mut length).expect("frame length");
    let mut bytes = vec![0_u8; usize::from(length[0])];
    stream.read_exact(&mut bytes).expect("frame bytes");
    bytes
}
