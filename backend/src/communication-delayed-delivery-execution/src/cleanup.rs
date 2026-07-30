use crate::{BodyCleanupPortV1, CleanupStorePortV1, ExecutionStoreErrorV1};

const BASE_RETRY_MILLIS: u64 = 250;
const MAX_RETRY_MILLIS: u64 = 5 * 60 * 1_000;
const MAX_BACKOFF_EXPONENT: u32 = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryCleanupOutcomeV1 {
    Idle,
    Completed,
    Rescheduled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryCleanupErrorV1 {
    InvalidInput,
    Store(ExecutionStoreErrorV1),
}

pub async fn process_body_cleanup_once_v1(
    store: &mut impl CleanupStorePortV1,
    cleanup_port: &mut impl BodyCleanupPortV1,
    logical_owner_id: &str,
    now_unix_millis: u64,
) -> Result<DelayedDeliveryCleanupOutcomeV1, DelayedDeliveryCleanupErrorV1> {
    if logical_owner_id.is_empty() || logical_owner_id.len() > 128 || now_unix_millis == 0 {
        return Err(DelayedDeliveryCleanupErrorV1::InvalidInput);
    }
    let Some(job) = store
        .next_pending_cleanup(logical_owner_id, now_unix_millis)
        .await
        .map_err(DelayedDeliveryCleanupErrorV1::Store)?
    else {
        return Ok(DelayedDeliveryCleanupOutcomeV1::Idle);
    };
    if cleanup_port.request_cleanup(&job).await.is_ok() {
        store
            .complete_cleanup(&job, now_unix_millis)
            .await
            .map_err(DelayedDeliveryCleanupErrorV1::Store)?;
        return Ok(DelayedDeliveryCleanupOutcomeV1::Completed);
    }
    let next_attempt_at_unix_millis = now_unix_millis
        .checked_add(retry_delay_millis(job.attempt_count))
        .ok_or(DelayedDeliveryCleanupErrorV1::InvalidInput)?;
    store
        .reschedule_cleanup(&job, next_attempt_at_unix_millis, now_unix_millis)
        .await
        .map_err(DelayedDeliveryCleanupErrorV1::Store)?;
    Ok(DelayedDeliveryCleanupOutcomeV1::Rescheduled)
}

fn retry_delay_millis(attempt_count: u32) -> u64 {
    BASE_RETRY_MILLIS
        .saturating_mul(1_u64 << attempt_count.min(MAX_BACKOFF_EXPONENT))
        .min(MAX_RETRY_MILLIS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BodyCleanupErrorV1, BodyCleanupReasonV1, DelayedDeliveryBodyCleanupJobV1,
        DelayedDeliveryBodyReceiptV1,
    };

    struct StoreFixture {
        job: Option<DelayedDeliveryBodyCleanupJobV1>,
        completed: bool,
        rescheduled_at: Option<u64>,
    }

    impl CleanupStorePortV1 for StoreFixture {
        async fn next_pending_cleanup(
            &mut self,
            _logical_owner_id: &str,
            _now_unix_millis: u64,
        ) -> Result<Option<DelayedDeliveryBodyCleanupJobV1>, ExecutionStoreErrorV1> {
            Ok(self.job.clone())
        }

        async fn complete_cleanup(
            &mut self,
            _job: &DelayedDeliveryBodyCleanupJobV1,
            _completed_at_unix_millis: u64,
        ) -> Result<(), ExecutionStoreErrorV1> {
            self.completed = true;
            Ok(())
        }

        async fn reschedule_cleanup(
            &mut self,
            _job: &DelayedDeliveryBodyCleanupJobV1,
            next_attempt_at_unix_millis: u64,
            _rescheduled_at_unix_millis: u64,
        ) -> Result<(), ExecutionStoreErrorV1> {
            self.rescheduled_at = Some(next_attempt_at_unix_millis);
            Ok(())
        }
    }

    struct CleanupFixture {
        unavailable: bool,
    }

    impl BodyCleanupPortV1 for CleanupFixture {
        async fn request_cleanup(
            &mut self,
            _job: &DelayedDeliveryBodyCleanupJobV1,
        ) -> Result<(), BodyCleanupErrorV1> {
            if self.unavailable {
                Err(BodyCleanupErrorV1::Unavailable)
            } else {
                Ok(())
            }
        }
    }

    fn job() -> DelayedDeliveryBodyCleanupJobV1 {
        DelayedDeliveryBodyCleanupJobV1 {
            logical_owner_id: "owner-1".to_owned(),
            delayed_operation_id: [1; 16],
            body_receipt: DelayedDeliveryBodyReceiptV1 {
                reference_id: [2; 16],
                declared_bytes: 3,
                sha256: [4; 32],
                custody_proof: vec![5; 64],
            },
            reason: BodyCleanupReasonV1::DeliveryCancelled,
            attempt_count: 0,
        }
    }

    #[tokio::test]
    async fn completes_only_after_release_authority_accepts() {
        let mut store = StoreFixture {
            job: Some(job()),
            completed: false,
            rescheduled_at: None,
        };
        assert_eq!(
            process_body_cleanup_once_v1(
                &mut store,
                &mut CleanupFixture { unavailable: false },
                "owner-1",
                1_000,
            )
            .await,
            Ok(DelayedDeliveryCleanupOutcomeV1::Completed)
        );
        assert!(store.completed);
        assert_eq!(store.rescheduled_at, None);
    }

    #[tokio::test]
    async fn persists_bounded_backoff_after_release_outage() {
        let mut store = StoreFixture {
            job: Some(job()),
            completed: false,
            rescheduled_at: None,
        };
        assert_eq!(
            process_body_cleanup_once_v1(
                &mut store,
                &mut CleanupFixture { unavailable: true },
                "owner-1",
                1_000,
            )
            .await,
            Ok(DelayedDeliveryCleanupOutcomeV1::Rescheduled)
        );
        assert!(!store.completed);
        assert_eq!(store.rescheduled_at, Some(1_250));
        assert_eq!(retry_delay_millis(32), MAX_RETRY_MILLIS);
    }
}
