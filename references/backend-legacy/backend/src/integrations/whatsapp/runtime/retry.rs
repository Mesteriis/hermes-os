use chrono::{DateTime, Utc};

use super::{RETRY_BASE_DELAY_SECONDS, RETRY_MAX_DELAY_SECONDS};

pub(super) fn next_attempt_at(
    now: DateTime<Utc>,
    retry_count: i32,
    retry_after_seconds: Option<i64>,
) -> DateTime<Utc> {
    now + chrono::Duration::seconds(retry_delay_seconds(retry_count, retry_after_seconds))
}

pub(super) fn retry_delay_seconds(retry_count: i32, retry_after_seconds: Option<i64>) -> i64 {
    if let Some(seconds) = retry_after_seconds {
        return seconds.clamp(1, RETRY_MAX_DELAY_SECONDS);
    }

    let retry_index = retry_count.saturating_sub(1).clamp(0, 20) as u32;
    let multiplier = 1_i64.checked_shl(retry_index).unwrap_or(i64::MAX);
    RETRY_BASE_DELAY_SECONDS
        .saturating_mul(multiplier)
        .min(RETRY_MAX_DELAY_SECONDS)
}
