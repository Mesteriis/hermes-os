//! Retention limits for diagnostic-only telemetry.

#[allow(dead_code)] // Used by the production retention preset.
pub const MAX_SEGMENT_BYTES: u64 = 1024 * 1024;
#[allow(dead_code)] // Used by the production retention preset.
pub const MAX_TOTAL_BYTES: u64 = 32 * 1024 * 1024;
#[allow(dead_code)] // Used by the production retention preset.
pub const MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug)]
pub struct TelemetryRetentionV1 {
    max_segment_bytes: u64,
    max_total_bytes: u64,
    max_age_seconds: u64,
}

impl TelemetryRetentionV1 {
    pub fn new(
        max_segment_bytes: u64,
        max_total_bytes: u64,
        max_age_seconds: u64,
    ) -> Result<Self, String> {
        if max_segment_bytes == 0 || max_segment_bytes > max_total_bytes || max_age_seconds == 0 {
            return Err("Telemetry retention policy is invalid".to_owned());
        }
        Ok(Self {
            max_segment_bytes,
            max_total_bytes,
            max_age_seconds,
        })
    }

    #[must_use]
    #[allow(dead_code)] // Used by production Collector composition.
    pub const fn production_default() -> Self {
        Self {
            max_segment_bytes: MAX_SEGMENT_BYTES,
            max_total_bytes: MAX_TOTAL_BYTES,
            max_age_seconds: MAX_AGE_SECONDS,
        }
    }

    #[must_use]
    pub const fn max_segment_bytes(self) -> u64 {
        self.max_segment_bytes
    }
    #[must_use]
    pub const fn max_total_bytes(self) -> u64 {
        self.max_total_bytes
    }
    #[must_use]
    pub const fn max_age_seconds(self) -> u64 {
        self.max_age_seconds
    }
}
