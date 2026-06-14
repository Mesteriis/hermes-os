use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domains::mail::core::{
    EmailProviderKind, ProviderAccountSecretPurpose, StoredRawCommunicationRecord,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncPlan {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub credential_purpose: ProviderAccountSecretPurpose,
    pub stream_id: String,
    pub adapter_config: EmailSyncAdapterConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailSyncAdapterConfig {
    Gmail {
        history_stream_id: String,
    },
    Imap {
        host: String,
        port: u16,
        tls: bool,
        mailbox: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FetchedEmailMessage {
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBatch {
    pub provider_kind: EmailProviderKind,
    pub stream_id: String,
    pub checkpoint: Option<Value>,
    pub messages: Vec<FetchedEmailMessage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBlobImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
    pub blobs_upserted: usize,
    pub raw_records: Vec<StoredRawCommunicationRecord>,
}
