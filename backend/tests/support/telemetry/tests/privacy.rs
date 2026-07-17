use hermes_telemetry_protocol::{
    TelemetryIdentityErrorV1, TelemetryPriorityV1, TelemetrySignalErrorV1, TelemetrySignalKindV1,
    TelemetrySignalV1, TelemetrySourceV1,
};

fn source() -> TelemetrySourceV1 {
    TelemetrySourceV1::new("runtime-42".into(), "module.lifecycle".into()).unwrap()
}

#[test]
fn signal_only_accepts_allowlisted_technical_fields() {
    let signal = TelemetrySignalV1::new(
        1_000,
        source(),
        TelemetrySignalKindV1::Lifecycle,
        TelemetryPriorityV1::Info,
        "runtime.start".into(),
        None,
        Some("trace-42".into()),
        0,
    );
    assert!(signal.is_ok());
}

#[test]
fn source_identity_rejects_private_or_user_provided_text() {
    assert_eq!(
        TelemetrySourceV1::new("owner@example.test".into(), "module".into()),
        Err(TelemetryIdentityErrorV1::InvalidRuntimeId),
    );
}

#[test]
fn signal_rejects_private_content_in_every_text_field() {
    let invalid_operation = TelemetrySignalV1::new(
        1_000,
        source(),
        TelemetrySignalKindV1::Log,
        TelemetryPriorityV1::Error,
        "mail subject: private".into(),
        None,
        None,
        0,
    );
    assert_eq!(
        invalid_operation,
        Err(TelemetrySignalErrorV1::InvalidOperation)
    );
    let invalid_trace = TelemetrySignalV1::new(
        1_000,
        source(),
        TelemetrySignalKindV1::Trace,
        TelemetryPriorityV1::Info,
        "rpc.call".into(),
        None,
        Some("alice@example.test".into()),
        0,
    );
    assert_eq!(invalid_trace, Err(TelemetrySignalErrorV1::InvalidTraceId));
}
