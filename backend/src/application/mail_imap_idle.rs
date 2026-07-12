use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use serde_json::json;
use sqlx::PgPool;

use super::bootstrap::{
    ApplicationBootstrapContext, host_vault_is_unlocked, runtime_allows_processing,
};
use crate::application::mail_background_sync::{
    DEFAULT_GMAIL_API_BASE_URL, DEFAULT_MAIL_SYNC_BLOB_ROOT, MailBackgroundSyncService,
    MailImapIdleOutcome, MailSyncStore, MailSyncTrigger,
};
use crate::domains::communications::core::CommunicationProviderSecretBindingStore;
use crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort;
use crate::vault::HostVault;

static MAIL_IMAP_IDLE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const MAIL_IMAP_IDLE_RUNTIME: &str = "mail_imap_idle";
const MANAGER_TICK_SECONDS: u64 = 30;
const IDLE_TIMEOUT_SECONDS: u64 = 29 * 60;
const POLL_FALLBACK_SECONDS: u64 = 300;

pub(super) fn start(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let store = MailSyncStore::new(pool.clone());
        let service = MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(LiveEmailProviderSyncPort::new(
                pool.clone(),
                vault.clone(),
                Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
                DEFAULT_GMAIL_API_BASE_URL,
            )),
            Arc::new(
                crate::domains::communications::provider_resources::MailProviderResourceStore::new(
                    pool.clone(),
                ),
            ),
        );
        let mut workers = HashMap::<String, tokio::task::JoinHandle<()>>::new();
        let mut tick = tokio::time::interval(Duration::from_secs(MANAGER_TICK_SECONDS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            let account_ids = match store.imap_idle_account_ids(100).await {
                Ok(account_ids) => account_ids,
                Err(error) => {
                    tracing::warn!(error = %error, "mail IMAP IDLE account discovery failed");
                    continue;
                }
            };
            let eligible = account_ids.into_iter().collect::<HashSet<_>>();
            workers.retain(|account_id, worker| {
                if !eligible.contains(account_id) {
                    worker.abort();
                    return false;
                }
                !worker.is_finished()
            });

            for account_id in eligible {
                if workers.contains_key(&account_id) {
                    continue;
                }
                let worker = tokio::spawn(run_worker(
                    account_id.clone(),
                    service.clone(),
                    pool.clone(),
                    vault.clone(),
                ));
                workers.insert(account_id, worker);
            }
        }
    });
}

async fn run_worker(
    account_id: String,
    service: MailBackgroundSyncService,
    pool: PgPool,
    vault: HostVault,
) {
    let mut consecutive_failures = 0_u32;
    let mut unsupported_reported = false;
    loop {
        if !runtime_allows_processing(
            &pool,
            "mail",
            MAIL_IMAP_IDLE_RUNTIME,
            json!({ "label": "Mail IMAP IDLE", "scope": "scheduler" }),
        )
        .await
            || !host_vault_is_unlocked(&vault)
        {
            tokio::time::sleep(Duration::from_secs(MANAGER_TICK_SECONDS)).await;
            continue;
        }

        match service
            .wait_for_imap_change(&account_id, Duration::from_secs(IDLE_TIMEOUT_SECONDS))
            .await
        {
            Ok(MailImapIdleOutcome::Changed) => {
                consecutive_failures = 0;
                unsupported_reported = false;
                if service
                    .run_account(&account_id, MailSyncTrigger::Scheduled)
                    .await
                    .is_err()
                {
                    tracing::warn!(
                        account_id,
                        "IMAP IDLE observed a change but the polling sync trigger failed"
                    );
                }
            }
            Ok(MailImapIdleOutcome::TimedOut) => {
                consecutive_failures = 0;
                unsupported_reported = false;
            }
            Ok(MailImapIdleOutcome::Unsupported) => {
                consecutive_failures = 0;
                if !unsupported_reported {
                    tracing::info!(
                        account_id,
                        "IMAP server does not advertise IDLE; polling fallback remains active"
                    );
                    unsupported_reported = true;
                }
                tokio::time::sleep(Duration::from_secs(POLL_FALLBACK_SECONDS)).await;
            }
            Ok(MailImapIdleOutcome::Disabled) => return,
            Err(_) => {
                consecutive_failures = consecutive_failures.saturating_add(1);
                let delay = reconnect_delay(&account_id, consecutive_failures);
                tracing::warn!(
                    account_id,
                    consecutive_failures,
                    reconnect_delay_ms = delay.as_millis(),
                    "IMAP IDLE connection failed; polling fallback remains active"
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

fn reconnect_delay(account_id: &str, consecutive_failures: u32) -> Duration {
    let exponent = consecutive_failures.saturating_sub(1).min(6);
    let base_seconds = 5_u64.saturating_mul(1_u64 << exponent).min(300);
    let account_jitter = account_id
        .bytes()
        .fold(u64::from(consecutive_failures) * 137, |hash, byte| {
            hash.wrapping_mul(31).wrapping_add(u64::from(byte))
        })
        % 1_000;
    Duration::from_secs(base_seconds) + Duration::from_millis(account_jitter)
}

fn register_scheduler(database_url: &str) -> bool {
    match MAIL_IMAP_IDLE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail IMAP IDLE scheduler registry is unavailable");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn scheduler_registration_and_reconnect_backoff_are_bounded() {
        let database_url = format!(
            "postgres://imap-idle-scheduler-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        assert!(register_scheduler(&database_url));
        assert!(!register_scheduler(&database_url));

        let first = reconnect_delay("mail-account", 1);
        let second = reconnect_delay("mail-account", 2);
        let saturated = reconnect_delay("mail-account", 100);
        assert!(first >= Duration::from_secs(5));
        assert!(second >= Duration::from_secs(10));
        assert!(saturated >= Duration::from_secs(300));
        assert!(saturated < Duration::from_secs(301));
    }
}
