//! External-runtime access to a non-secret, fenced Storage binding.

mod binding;
mod credential_fence;

pub(crate) use binding::current_binding;
pub(crate) use credential_fence::validate_vault_credential_fence;
