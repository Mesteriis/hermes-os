pub(crate) mod client;
pub(crate) mod folder_requests;
mod identifiers;
mod library_paths;
pub(crate) mod parsing;
pub(crate) mod qr_login;
pub(crate) mod qr_login_support;
pub(crate) mod requests;
pub(crate) mod snapshots;
#[cfg(test)]
use self::library_paths::{tdjson_library_candidates_with_context, tdjson_platform_dir};
#[cfg(test)]
use self::qr_login::commands::cancel_existing_qr_logins_for_account;
#[cfg(test)]
use self::qr_login_support::completion::{mark_worker_complete, new_worker_completion};

#[cfg(test)]
mod tests;
