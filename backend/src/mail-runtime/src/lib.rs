//! Typed Mail managed-runtime admission contract.

pub mod client_port;
pub mod communications_outbox;
pub mod managed;
pub mod settings;

use hermes_mail_api::MailAccountConfigurationV1;

#[derive(Clone)]
pub struct MailCredentialRevisionsV1 {
    pub imap_password: Option<u64>,
    pub gmail_access_token: Option<u64>,
    pub smtp_password: Option<u64>,
}

#[derive(Clone)]
pub struct MailRuntimeAdmission {
    pub logical_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
    pub account: MailAccountConfigurationV1,
    pub credential_revisions: MailCredentialRevisionsV1,
}
