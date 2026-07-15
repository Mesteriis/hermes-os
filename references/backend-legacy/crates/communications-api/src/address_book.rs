use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::accounts::CommunicationProviderKind;
use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderFetchRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_config: Value,
    pub page_token: Option<String>,
    pub page_size: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderUpsertRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_address_book_entry_id: Option<String>,
    pub provider_etag: Option<String>,
    pub display_name: String,
    pub email_address: Option<String>,
    pub phone_numbers: Vec<String>,
    pub remote_write_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderEntry {
    pub provider_address_book_entry_id: String,
    pub display_name: Option<String>,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookProviderBatch {
    pub entries: Vec<AddressBookProviderEntry>,
    pub next_page_token: Option<String>,
}

pub type AddressBookProviderSyncFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AddressBookProviderSyncError>> + Send + 'a>>;

pub type SharedAddressBookProviderSyncPort = Arc<dyn AddressBookProviderSyncPort>;

pub trait AddressBookProviderSyncPort: Send + Sync {
    fn fetch_entries<'a>(
        &'a self,
        request: AddressBookProviderFetchRequest,
    ) -> AddressBookProviderSyncFuture<'a, AddressBookProviderBatch>;

    fn upsert_entry<'a>(
        &'a self,
        request: AddressBookProviderUpsertRequest,
    ) -> AddressBookProviderSyncFuture<'a, AddressBookProviderEntry>;
}

#[derive(Debug, Error)]
pub enum AddressBookProviderSyncError {
    #[error("address book sync is not supported for provider: {0}")]
    UnsupportedProvider(String),
    #[error("provider credential error: {0}")]
    Credential(String),
    #[error("remote address book write is blocked: {0}")]
    RemoteWriteBlocked(&'static str),
    #[error("provider network error: {0}")]
    ProviderNetwork(String),
}
