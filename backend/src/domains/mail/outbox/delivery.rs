use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

use super::{EmailOutboxError, EmailOutboxItem, EmailOutboxStore};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxSendReceipt {
    pub provider_message_id: String,
    pub accepted_recipients: Vec<String>,
}

#[derive(Debug, Error)]
pub enum OutboxDeliveryError {
    #[error("{0}")]
    Transport(String),
}

impl OutboxDeliveryError {
    pub fn public_message(&self) -> &str {
        match self {
            Self::Transport(message) => message.as_str(),
        }
    }
}

pub trait OutboxEmailSender: Send + Sync {
    fn send<'a>(
        &'a self,
        item: &'a EmailOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRetryPolicy {
    max_attempts: i32,
    base_delay: Duration,
    max_delay: Duration,
}

impl OutboxRetryPolicy {
    pub fn new(max_attempts: i32, base_delay: Duration, max_delay: Duration) -> Self {
        let base_delay = duration_with_minimum_seconds(base_delay, 1);
        let max_delay = duration_with_minimum_seconds(max_delay, base_delay.num_seconds());

        Self {
            max_attempts: max_attempts.max(1),
            base_delay,
            max_delay,
        }
    }

    pub fn disabled() -> Self {
        Self::new(1, Duration::seconds(1), Duration::seconds(1))
    }

    fn next_attempt_at(
        &self,
        now: DateTime<Utc>,
        completed_attempts: i32,
    ) -> Option<DateTime<Utc>> {
        if completed_attempts >= self.max_attempts {
            return None;
        }

        let retry_index = completed_attempts.saturating_sub(1).max(0) as u32;
        let factor = 1_i64.checked_shl(retry_index.min(30)).unwrap_or(i64::MAX);
        let delay_seconds = self
            .base_delay
            .num_seconds()
            .saturating_mul(factor)
            .min(self.max_delay.num_seconds());

        Some(now + Duration::seconds(delay_seconds))
    }
}

impl Default for OutboxRetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::seconds(30), Duration::minutes(15))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxDeliveryReport {
    pub claimed: usize,
    pub sent: usize,
    pub failed: usize,
    pub retried: usize,
}

#[derive(Clone)]
pub struct EmailOutboxDeliveryWorker<S> {
    store: EmailOutboxStore,
    sender: S,
    retry_policy: OutboxRetryPolicy,
}

impl<S> EmailOutboxDeliveryWorker<S>
where
    S: OutboxEmailSender,
{
    pub fn new(store: EmailOutboxStore, sender: S) -> Self {
        Self::with_retry_policy(store, sender, OutboxRetryPolicy::default())
    }

    pub fn with_retry_policy(
        store: EmailOutboxStore,
        sender: S,
        retry_policy: OutboxRetryPolicy,
    ) -> Self {
        Self {
            store,
            sender,
            retry_policy,
        }
    }

    pub async fn deliver_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<OutboxDeliveryReport, EmailOutboxError> {
        let claimed = self.store.claim_due(now, limit).await?;
        let mut report = OutboxDeliveryReport {
            claimed: claimed.len(),
            sent: 0,
            failed: 0,
            retried: 0,
        };

        for item in claimed {
            match self.sender.send(&item).await {
                Ok(receipt) => {
                    self.store.mark_sent(&item.outbox_id, now, &receipt).await?;
                    report.sent += 1;
                }
                Err(error) => {
                    if let Some(next_attempt_at) =
                        self.retry_policy.next_attempt_at(now, item.send_attempts)
                    {
                        self.store
                            .mark_retry_scheduled(
                                &item.outbox_id,
                                now,
                                next_attempt_at,
                                error.public_message(),
                            )
                            .await?;
                        report.retried += 1;
                    } else {
                        self.store
                            .mark_failed(&item.outbox_id, now, error.public_message())
                            .await?;
                        report.failed += 1;
                    }
                }
            }
        }

        Ok(report)
    }
}

fn duration_with_minimum_seconds(duration: Duration, minimum_seconds: i64) -> Duration {
    Duration::seconds(duration.num_seconds().max(minimum_seconds))
}
