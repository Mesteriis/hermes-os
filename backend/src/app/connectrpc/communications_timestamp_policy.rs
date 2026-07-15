use chrono::{DateTime, Utc};
use connectrpc::{ConnectError, ErrorCode};

pub(super) fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, ConnectError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| {
            ConnectError::new(
                ErrorCode::InvalidArgument,
                format!("invalid RFC3339 timestamp: {value}"),
            )
        })
}
