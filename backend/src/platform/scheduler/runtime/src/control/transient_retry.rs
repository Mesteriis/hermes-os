//! Bounded backoff for transient Scheduler infrastructure failures.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MAX_CONSECUTIVE_FAILURES: u8 = 10;
const BASE_BACKOFF_MILLIS: u64 = 25;
const MAX_BACKOFF_MILLIS: u64 = 500;
const JITTER_WINDOW_MILLIS: u64 = 25;

static JITTER_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
pub(super) struct SchedulerTransientRetryV1 {
    consecutive_failures: u8,
}

impl SchedulerTransientRetryV1 {
    pub(super) fn reset(&mut self) {
        self.consecutive_failures = 0;
    }

    pub(super) fn next_delay(&mut self) -> Option<Duration> {
        if self.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            return None;
        }
        self.consecutive_failures += 1;
        let shift = u32::from(self.consecutive_failures.saturating_sub(1).min(5));
        let exponential = BASE_BACKOFF_MILLIS
            .checked_shl(shift)
            .unwrap_or(MAX_BACKOFF_MILLIS)
            .min(MAX_BACKOFF_MILLIS);
        let sequence = JITTER_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let jitter = sequence
            .wrapping_mul(17)
            .wrapping_add(u64::from(self.consecutive_failures) * 13)
            % JITTER_WINDOW_MILLIS;
        Some(Duration::from_millis(exponential + jitter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_retry_is_bounded_and_resets_after_progress() {
        let mut retry = SchedulerTransientRetryV1::default();
        let delays = (0..MAX_CONSECUTIVE_FAILURES)
            .map(|_| retry.next_delay().expect("retry budget"))
            .collect::<Vec<_>>();
        assert!(retry.next_delay().is_none());
        assert!(
            delays
                .iter()
                .all(|delay| *delay >= Duration::from_millis(BASE_BACKOFF_MILLIS))
        );
        assert!(delays.iter().all(|delay| {
            *delay < Duration::from_millis(MAX_BACKOFF_MILLIS + JITTER_WINDOW_MILLIS)
        }));

        retry.reset();
        assert!(retry.next_delay().is_some());
    }

    #[test]
    fn transient_retry_grows_before_reaching_its_cap() {
        let mut retry = SchedulerTransientRetryV1::default();
        let first = retry.next_delay().expect("first retry");
        let second = retry.next_delay().expect("second retry");
        let third = retry.next_delay().expect("third retry");
        assert!(second > first);
        assert!(third > second);
    }
}
