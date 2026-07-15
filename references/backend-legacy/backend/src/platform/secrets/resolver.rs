use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::validation::validate_secret_resolution_ref;

pub type SecretResolutionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ResolvedSecret, SecretResolutionError>> + Send + 'a>>;

pub trait SecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemorySecretResolver {
    values: HashMap<String, ResolvedSecret>,
}

impl InMemorySecretResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        secret_ref: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), SecretResolutionError> {
        let secret_ref = secret_ref.into();
        validate_secret_resolution_ref(&secret_ref)?;
        let resolved_secret = ResolvedSecret::new(value)?;

        self.values
            .insert(secret_ref.trim().to_owned(), resolved_secret);

        Ok(())
    }

    fn resolve_reference(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::TestDouble {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        validate_secret_resolution_ref(&reference.secret_ref)?;
        let secret_ref = reference.secret_ref.trim();

        self.values
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            })
    }
}

impl SecretResolver for InMemorySecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_reference(reference)))
    }
}
