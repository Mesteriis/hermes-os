//! Explicit timezone context; the Clock never derives local civil time.

pub const MAX_TIME_ZONE_NAME_BYTES: usize = 128;
pub const MAX_UTC_OFFSET_SECONDS: i32 = 86_400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeZoneContextV1 {
    iana_name: String,
    utc_offset_seconds: i32,
}

impl TimeZoneContextV1 {
    pub fn new(iana_name: String, utc_offset_seconds: i32) -> Result<Self, TimeZoneContextErrorV1> {
        if !is_iana_name(&iana_name) {
            return Err(TimeZoneContextErrorV1::InvalidIanaName);
        }
        if !(-MAX_UTC_OFFSET_SECONDS..=MAX_UTC_OFFSET_SECONDS).contains(&utc_offset_seconds) {
            return Err(TimeZoneContextErrorV1::OffsetOutOfRange);
        }
        Ok(Self {
            iana_name,
            utc_offset_seconds,
        })
    }

    #[must_use]
    pub fn iana_name(&self) -> &str {
        &self.iana_name
    }

    #[must_use]
    pub const fn utc_offset_seconds(&self) -> i32 {
        self.utc_offset_seconds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeZoneContextErrorV1 {
    InvalidIanaName,
    OffsetOutOfRange,
}

fn is_iana_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TIME_ZONE_NAME_BYTES
        && value.contains('/')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+' | b'.')
        })
}
