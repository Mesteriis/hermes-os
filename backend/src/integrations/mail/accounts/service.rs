mod constructors;
mod gmail_complete;
mod gmail_payloads;
mod gmail_refresh;
mod gmail_start;
mod imap;
mod imap_payloads;
mod stores;
mod token_http;

use reqwest::Client;
use sqlx::postgres::PgPool;

use crate::platform::secrets::SecretReferenceStore;

use super::vault::AccountSecretVault;

#[derive(Clone)]
pub struct EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) pool: Option<PgPool>,
    pub(in crate::integrations::mail::accounts::service) secret_store: Option<SecretReferenceStore>,
    pub(in crate::integrations::mail::accounts::service) vault: AccountSecretVault,
    pub(in crate::integrations::mail::accounts::service) http: Client,
}
