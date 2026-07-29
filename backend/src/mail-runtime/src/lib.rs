//! Typed Mail managed-runtime admission contract.

pub mod account_lifecycle;
pub mod admission;
pub mod attachment_anchor_mapping;
pub mod attachment_safety_projection;
pub mod attachment_security_outbox;
pub mod client_port;
pub mod communications_outbox;
pub mod delivery_intent_consumer;
pub mod delivery_intent_execution;
pub mod delivery_intent_outbox;
pub mod delivery_intent_result;
pub mod delivery_intent_worker;
pub mod gmail_oauth;
pub mod gmail_sync_worker;
pub mod managed;
pub mod settings;

use hermes_mail_api::{GmailOAuthConfigurationV1, MailAccountConfigurationV1};

#[derive(Clone)]
pub struct MailRuntimeAdmission {
    pub logical_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
    pub settings_revision: u64,
    pub account: MailAccountConfigurationV1,
    pub gmail_oauth: Option<GmailOAuthConfigurationV1>,
}
