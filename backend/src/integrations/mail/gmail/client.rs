mod errors;
mod gmail_api;
mod helpers;
mod imap;
mod models;
mod options;

pub use errors::EmailProviderNetworkError;
pub use gmail_api::GmailApiClient;
pub use imap::{ImapIdleOutcome, ImapMailboxDescriptor, ImapMailboxRole, ImapNetworkClient};
pub use options::{
    GmailContactFetchOptions, GmailFetchOptions, GmailHistoryFetchOptions, ImapFetchOptions,
    ImapIdleOptions, ImapMailboxListOptions,
};
