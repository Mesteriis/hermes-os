pub const DEFAULT_MAIL_SYNC_BATCH_SIZE: i32 = 100;
pub const DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS: i32 = 300;

const MAX_BATCH_SIZE: i32 = 500;
const MIN_POLL_INTERVAL_SECONDS: i32 = 60;
const MAX_POLL_INTERVAL_SECONDS: i32 = 86_400;
pub const DEFAULT_GMAIL_API_BASE_URL: &str = "https://www.googleapis.com";

pub(crate) mod errors;
mod events;
mod evidence;
pub(crate) mod idle;
pub(crate) mod models;
mod provider;
mod rows;
pub(crate) mod service;
pub mod store;
mod validation;
