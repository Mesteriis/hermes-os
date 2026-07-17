//! Stable opaque telemetry source identity.

pub const MAX_TELEMETRY_SOURCE_BYTES: usize = 128;
pub const MAX_TELEMETRY_COMPONENT_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetrySourceV1 {
    runtime_id: String,
    component: String,
}

impl TelemetrySourceV1 {
    pub fn new(runtime_id: String, component: String) -> Result<Self, TelemetryIdentityErrorV1> {
        validate_identifier(&runtime_id, MAX_TELEMETRY_SOURCE_BYTES)
            .map_err(|_| TelemetryIdentityErrorV1::InvalidRuntimeId)?;
        validate_identifier(&component, MAX_TELEMETRY_COMPONENT_BYTES)
            .map_err(|_| TelemetryIdentityErrorV1::InvalidComponent)?;
        Ok(Self {
            runtime_id,
            component,
        })
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryIdentityErrorV1 {
    InvalidRuntimeId,
    InvalidComponent,
}

fn validate_identifier(value: &str, maximum_bytes: usize) -> Result<(), ()> {
    if value.is_empty() || value.len() > maximum_bytes {
        return Err(());
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Ok(());
    }
    Err(())
}
