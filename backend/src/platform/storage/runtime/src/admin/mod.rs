//! Storage-owned infrastructure administration composition.

mod bindings;
mod migrations;
mod postgres;
mod readiness;

pub(crate) use bindings::{apply_authorized_bindings, apply_staged_pool_configuration};
pub(crate) use migrations::apply_authorized_migrations;
pub(crate) use postgres::{
    RuntimeRoleCredentialV1, connect_platform, reconcile_authorized_roles, verify_platform_postgres,
};
pub(crate) use readiness::{admin_credential, admin_endpoint, verify_platform_admin};
