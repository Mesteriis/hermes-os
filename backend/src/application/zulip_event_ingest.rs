use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, CommunicationProviderAccountStore,
    CommunicationProviderKind, CommunicationProviderSecretBindingStore, NewIngestionCheckpoint,
    ProviderAccount, ProviderAccountSecretPurpose, ProviderCredentialReader,
};
use crate::domains::signal_hub::{SignalHubError, dispatch_zulip_raw_signal};
use crate::integrations::zulip::{
    ZulipApiClient, ZulipClientConfig, ZulipClientError, ZulipEventMappingContext,
    map_zulip_event_to_raw_record,
};
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

const ZULIP_EVENT_QUEUE_STREAM_ID: &str = "zulip:event_queue";
const ZULIP_EVENT_TYPES: &[&str] = &["message", "reaction", "update_message", "delete_message"];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ZulipEventIngestReport {
    pub accounts_scanned: usize,
    pub accounts_failed: usize,
    pub queues_registered: usize,
    pub events_received: usize,
    pub raw_records_recorded: usize,
    pub accepted_signals: usize,
    pub checkpoints_saved: usize,
}

pub struct ZulipEventIngestWorker<R> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    ingestion_store: CommunicationIngestionStore,
    resolver: R,
}

impl<R> ZulipEventIngestWorker<R>
where
    R: SecretResolver,
{
    pub fn new(pool: PgPool, resolver: R) -> Self {
        Self {
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool.clone()),
            ingestion_store: CommunicationIngestionStore::new(pool.clone()),
            pool,
            resolver,
        }
    }
}

impl<R> ZulipEventIngestWorker<R>
where
    R: SecretResolver + Send + Sync,
{
    pub async fn poll_due(
        &self,
        now: DateTime<Utc>,
    ) -> Result<ZulipEventIngestReport, ZulipEventIngestWorkerError> {
        let accounts = self.provider_account_store.list().await?;
        let mut report = ZulipEventIngestReport::default();

        for account in accounts
            .into_iter()
            .filter(|account| account.provider_kind == CommunicationProviderKind::ZulipBot)
        {
            report.accounts_scanned += 1;
            match self.poll_account(&account.account_id, now).await {
                Ok(account_report) => report.merge(account_report),
                Err(error) => {
                    report.accounts_failed += 1;
                    tracing::warn!(
                        error = %error,
                        account_id = %account.account_id,
                        "zulip event ingest account tick failed"
                    );
                }
            }
        }

        Ok(report)
    }

    pub async fn poll_account(
        &self,
        account_id: &str,
        now: DateTime<Utc>,
    ) -> Result<ZulipEventIngestReport, ZulipEventIngestWorkerError> {
        let account = self.zulip_account(account_id).await?;
        let base_url = zulip_base_url(&account)?;
        let client = self.zulip_client(&account, base_url).await?;
        let mut queue_state = self.queue_state(&account.account_id).await?;
        let mut report = ZulipEventIngestReport {
            accounts_scanned: 1,
            ..ZulipEventIngestReport::default()
        };

        if queue_state.is_none() {
            queue_state = Some(self.register_queue(&client, &account.account_id).await?);
            report.queues_registered += 1;
            report.checkpoints_saved += 1;
        }

        let Some(mut queue_state) = queue_state else {
            return Ok(report);
        };

        let events_response = match client
            .get_events(&queue_state.queue_id, queue_state.last_event_id, true)
            .await
        {
            Ok(response) => response,
            Err(ZulipClientError::Api { status: 400, .. }) => {
                self.ingestion_store
                    .delete_checkpoint(&account.account_id, ZULIP_EVENT_QUEUE_STREAM_ID)
                    .await?;
                let registered = self.register_queue(&client, &account.account_id).await?;
                report.queues_registered += 1;
                report.checkpoints_saved += 1;
                let response = client
                    .get_events(&registered.queue_id, registered.last_event_id, true)
                    .await
                    .map_err(|error| client_error(&account.account_id, error))?;
                queue_state = registered;
                response
            }
            Err(error) => return Err(client_error(&account.account_id, error)),
        };

        if let Some(response_queue_id) = events_response
            .queue_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            queue_state.queue_id = response_queue_id.to_owned();
        }

        for event in events_response.events {
            queue_state.last_event_id = queue_state.last_event_id.max(event.id);
            report.events_received += 1;

            let mapping_context = ZulipEventMappingContext::new(&account.account_id, base_url, now)
                .with_import_batch_id(format!("zulip-event-queue:{}", queue_state.queue_id));
            let new_raw_record = map_zulip_event_to_raw_record(&event, &mapping_context)?;
            let raw_record = self
                .ingestion_store
                .record_raw_source(&new_raw_record)
                .await?;
            report.raw_records_recorded += 1;

            if dispatch_zulip_raw_signal(self.pool.clone(), &raw_record)
                .await?
                .is_some()
            {
                report.accepted_signals += 1;
            }
        }

        self.save_queue_state(&account.account_id, &queue_state)
            .await?;
        report.checkpoints_saved += 1;

        Ok(report)
    }

    async fn zulip_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, ZulipEventIngestWorkerError> {
        let account = self
            .provider_account_store
            .get(account_id)
            .await?
            .ok_or_else(|| ZulipEventIngestWorkerError::AccountNotFound {
                account_id: account_id.trim().to_owned(),
            })?;
        if account.provider_kind != CommunicationProviderKind::ZulipBot {
            return Err(ZulipEventIngestWorkerError::UnsupportedProvider {
                account_id: account.account_id,
                provider_kind: account.provider_kind.as_str(),
            });
        }
        Ok(account)
    }

    async fn zulip_client(
        &self,
        account: &ProviderAccount,
        base_url: &str,
    ) -> Result<ZulipApiClient, ZulipEventIngestWorkerError> {
        let credential_reader = ProviderCredentialReader::new(
            self.provider_secret_binding_store.clone(),
            self.secret_store.clone(),
            &self.resolver,
        );
        let credential = credential_reader
            .read(
                &account.account_id,
                ProviderAccountSecretPurpose::ZulipApiKey,
            )
            .await
            .map_err(|_| ZulipEventIngestWorkerError::CredentialUnavailable {
                account_id: account.account_id.clone(),
            })?;
        Ok(ZulipApiClient::new(
            ZulipClientConfig::new(
                base_url,
                account.external_account_id.as_str(),
                credential.secret.expose_for_runtime(),
            )
            .map_err(|_| ZulipEventIngestWorkerError::InvalidAccountConfig {
                account_id: account.account_id.clone(),
                field: "base_url",
            })?,
        ))
    }

    async fn queue_state(
        &self,
        account_id: &str,
    ) -> Result<Option<ZulipEventQueueState>, ZulipEventIngestWorkerError> {
        self.ingestion_store
            .checkpoint(account_id, ZULIP_EVENT_QUEUE_STREAM_ID)
            .await?
            .map(|checkpoint| queue_state_from_checkpoint(account_id, &checkpoint.checkpoint))
            .transpose()
    }

    async fn register_queue(
        &self,
        client: &ZulipApiClient,
        account_id: &str,
    ) -> Result<ZulipEventQueueState, ZulipEventIngestWorkerError> {
        let response = client
            .register_event_queue(ZULIP_EVENT_TYPES)
            .await
            .map_err(|error| client_error(account_id, error))?;
        let state = ZulipEventQueueState {
            queue_id: response.queue_id,
            last_event_id: response.last_event_id,
        };
        self.save_queue_state(account_id, &state).await?;
        Ok(state)
    }

    async fn save_queue_state(
        &self,
        account_id: &str,
        state: &ZulipEventQueueState,
    ) -> Result<(), ZulipEventIngestWorkerError> {
        self.ingestion_store
            .save_checkpoint(&NewIngestionCheckpoint::new(
                account_id,
                ZULIP_EVENT_QUEUE_STREAM_ID,
                json!({
                    "queue_id": state.queue_id,
                    "last_event_id": state.last_event_id,
                }),
            ))
            .await?;
        Ok(())
    }
}

impl ZulipEventIngestReport {
    fn merge(&mut self, other: Self) {
        self.queues_registered += other.queues_registered;
        self.events_received += other.events_received;
        self.raw_records_recorded += other.raw_records_recorded;
        self.accepted_signals += other.accepted_signals;
        self.checkpoints_saved += other.checkpoints_saved;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ZulipEventQueueState {
    queue_id: String,
    last_event_id: i64,
}

fn queue_state_from_checkpoint(
    account_id: &str,
    checkpoint: &Value,
) -> Result<ZulipEventQueueState, ZulipEventIngestWorkerError> {
    let queue_id = checkpoint
        .get("queue_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ZulipEventIngestWorkerError::InvalidQueueCheckpoint {
            account_id: account_id.to_owned(),
            field: "queue_id",
        })?;
    let last_event_id = checkpoint
        .get("last_event_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| ZulipEventIngestWorkerError::InvalidQueueCheckpoint {
            account_id: account_id.to_owned(),
            field: "last_event_id",
        })?;

    Ok(ZulipEventQueueState {
        queue_id: queue_id.to_owned(),
        last_event_id,
    })
}

fn zulip_base_url(account: &ProviderAccount) -> Result<&str, ZulipEventIngestWorkerError> {
    account
        .config
        .get("base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZulipEventIngestWorkerError::InvalidAccountConfig {
            account_id: account.account_id.clone(),
            field: "base_url",
        })
}

fn client_error(account_id: &str, error: ZulipClientError) -> ZulipEventIngestWorkerError {
    match error {
        ZulipClientError::Api { status, .. } => ZulipEventIngestWorkerError::ProviderApi {
            account_id: account_id.to_owned(),
            status,
        },
        ZulipClientError::InvalidRequest(_) => ZulipEventIngestWorkerError::InvalidClientRequest {
            account_id: account_id.to_owned(),
        },
        ZulipClientError::Json(_) => ZulipEventIngestWorkerError::InvalidProviderResponse {
            account_id: account_id.to_owned(),
        },
        ZulipClientError::Http(_) => ZulipEventIngestWorkerError::Transport {
            account_id: account_id.to_owned(),
        },
        ZulipClientError::Url(_) => ZulipEventIngestWorkerError::InvalidAccountConfig {
            account_id: account_id.to_owned(),
            field: "base_url",
        },
    }
}

#[derive(Debug, Error)]
pub enum ZulipEventIngestWorkerError {
    #[error("Zulip provider account `{account_id}` was not found")]
    AccountNotFound { account_id: String },
    #[error("provider account `{account_id}` is `{provider_kind}`, not `zulip_bot`")]
    UnsupportedProvider {
        account_id: String,
        provider_kind: &'static str,
    },
    #[error("Zulip provider account `{account_id}` has invalid `{field}` config")]
    InvalidAccountConfig {
        account_id: String,
        field: &'static str,
    },
    #[error("Zulip event queue checkpoint for account `{account_id}` has invalid `{field}`")]
    InvalidQueueCheckpoint {
        account_id: String,
        field: &'static str,
    },
    #[error("Zulip API credential is unavailable for account `{account_id}`")]
    CredentialUnavailable { account_id: String },
    #[error("Zulip API returned HTTP {status} for account `{account_id}`")]
    ProviderApi { account_id: String, status: u16 },
    #[error("Zulip event request was invalid for account `{account_id}`")]
    InvalidClientRequest { account_id: String },
    #[error("Zulip HTTP request failed for account `{account_id}`")]
    Transport { account_id: String },
    #[error("Zulip API response was invalid for account `{account_id}`")]
    InvalidProviderResponse { account_id: String },
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
    #[error(transparent)]
    Mapping(#[from] crate::integrations::zulip::ZulipEventMappingError),
    #[error(transparent)]
    SignalHub(#[from] SignalHubError),
}
