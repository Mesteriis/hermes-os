//! Fixed-shape telemetry signal without arbitrary payload fields.

use super::identity::TelemetrySourceV1;

pub const MAX_OPERATION_BYTES: usize = 96;
pub const MAX_ERROR_CLASS_BYTES: usize = 96;
pub const MAX_TRACE_ID_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetrySignalKindV1 {
    Log,
    Metric,
    Trace,
    Lifecycle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TelemetryPriorityV1 {
    Debug,
    Info,
    Warning,
    Error,
    Crash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetrySignalV1 {
    observed_at_utc_millis: i64,
    source: TelemetrySourceV1,
    kind: TelemetrySignalKindV1,
    priority: TelemetryPriorityV1,
    operation: String,
    error_class: Option<String>,
    trace_id: Option<String>,
    dropped_count: u32,
}

impl TelemetrySignalV1 {
    pub fn new(
        observed_at_utc_millis: i64,
        source: TelemetrySourceV1,
        kind: TelemetrySignalKindV1,
        priority: TelemetryPriorityV1,
        operation: String,
        error_class: Option<String>,
        trace_id: Option<String>,
        dropped_count: u32,
    ) -> Result<Self, TelemetrySignalErrorV1> {
        validate_field(
            &operation,
            MAX_OPERATION_BYTES,
            TelemetrySignalErrorV1::InvalidOperation,
        )?;
        validate_optional_field(
            &error_class,
            MAX_ERROR_CLASS_BYTES,
            TelemetrySignalErrorV1::InvalidErrorClass,
        )?;
        validate_optional_field(
            &trace_id,
            MAX_TRACE_ID_BYTES,
            TelemetrySignalErrorV1::InvalidTraceId,
        )?;
        Ok(Self {
            observed_at_utc_millis,
            source,
            kind,
            priority,
            operation,
            error_class,
            trace_id,
            dropped_count,
        })
    }

    #[must_use]
    pub const fn observed_at_utc_millis(&self) -> i64 {
        self.observed_at_utc_millis
    }
    #[must_use]
    pub fn source(&self) -> &TelemetrySourceV1 {
        &self.source
    }
    #[must_use]
    pub const fn kind(&self) -> TelemetrySignalKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn priority(&self) -> TelemetryPriorityV1 {
        self.priority
    }
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn error_class(&self) -> Option<&str> {
        self.error_class.as_deref()
    }
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }
    #[must_use]
    pub const fn dropped_count(&self) -> u32 {
        self.dropped_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetrySignalErrorV1 {
    InvalidOperation,
    InvalidErrorClass,
    InvalidTraceId,
}

fn validate_optional_field(
    value: &Option<String>,
    maximum_bytes: usize,
    error: TelemetrySignalErrorV1,
) -> Result<(), TelemetrySignalErrorV1> {
    value
        .as_deref()
        .map_or(Ok(()), |text| validate_field(text, maximum_bytes, error))
}

fn validate_field(
    value: &str,
    maximum_bytes: usize,
    error: TelemetrySignalErrorV1,
) -> Result<(), TelemetrySignalErrorV1> {
    if value.is_empty() || value.len() > maximum_bytes {
        return Err(error);
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
    {
        return Ok(());
    }
    Err(error)
}
