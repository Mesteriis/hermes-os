use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type PersonaOwnerQueryFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<String>, PersonaOwnerQueryError>> + Send + 'a>>;

pub trait PersonaOwnerQuery: Send + Sync {
    fn owner_language<'a>(&'a self) -> PersonaOwnerQueryFuture<'a>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonaRead {
    pub persona_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
    pub persona_type: String,
    pub is_self: bool,
    pub is_address_book: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type PersonaReadFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<PersonaRead>, PersonaQueryError>> + Send + 'a>>;
pub type PersonaListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<PersonaRead>, PersonaQueryError>> + Send + 'a>>;

pub trait PersonaReadPort: Send + Sync {
    fn list<'a>(&'a self, limit: i64) -> PersonaListFuture<'a>;
    fn get<'a>(&'a self, persona_id: &'a str) -> PersonaReadFuture<'a>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonaIdentityProjection {
    pub identity_type: String,
    pub identity_value: String,
    pub metadata: serde_json::Value,
}

pub type PersonaIdentityProjectionListFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Vec<PersonaIdentityProjection>, PersonaQueryError>> + Send + 'a>,
>;

pub trait PersonaIdentityProjectionPort: Send + Sync {
    fn list_for_values<'a>(
        &'a self,
        identity_type: &'a str,
        identity_values: &'a [String],
    ) -> PersonaIdentityProjectionListFuture<'a>;
}

pub struct PersonaUpdateCommand {
    pub persona_id: String,
    pub display_name: Option<String>,
    pub assign_owner: bool,
}

pub type PersonaWriteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<PersonaRead, PersonaWriteError>> + Send + 'a>>;

pub trait PersonaWritePort: Send + Sync {
    fn update<'a>(&'a self, command: PersonaUpdateCommand) -> PersonaWriteFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
#[error("persona query failed: {0}")]
pub struct PersonaQueryError(pub String);

#[derive(Debug, thiserror::Error)]
#[error("persona owner query failed: {0}")]
pub struct PersonaOwnerQueryError(pub String);

#[derive(Debug, thiserror::Error)]
#[error("persona write failed: {0}")]
pub struct PersonaWriteError(pub String);
