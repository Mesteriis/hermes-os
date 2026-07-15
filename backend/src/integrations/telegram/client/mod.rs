mod accounts;
mod chat_metadata;
mod chat_reconciliation;
pub mod chat_state;
mod chats;
pub mod commands;
pub(crate) mod errors;
mod evidence;
pub mod fixture_port;
pub(crate) mod identifiers;
pub mod lifecycle;
pub(crate) mod messages;
pub mod models;
mod observations;
pub mod participants;
pub mod reactions;
pub mod references;
pub mod rows;
mod search;
pub mod store;
#[cfg(test)]
mod tests;
pub mod topics;
mod validation;
pub(crate) mod vault;

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";
const TELEGRAM_CHAT_RECORD_KIND: &str = "telegram_chat";
const TELEGRAM_ACCOUNT_ACTIVE: &str = "active";
const TELEGRAM_ACCOUNT_LOGGED_OUT: &str = "logged_out";
const TELEGRAM_ACCOUNT_REMOVED: &str = "removed";

pub type ProviderCommunicationMessage = self::models::messages::TelegramMessage;
