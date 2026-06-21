use crate::domains::communications::core::{
    CommunicationProviderSecretBindingStore, ProviderCredentialError, ProviderCredentialReader,
};
use crate::integrations::telegram::client::TelegramError;
use crate::platform::communications::ProviderAccountSecretPurpose;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

pub(in crate::integrations::telegram::runtime) async fn optional_telegram_session_key(
    binding_store: &CommunicationProviderSecretBindingStore,
    secret_store: &SecretReferenceStore,
    secret_resolver: &(impl SecretResolver + Sync + ?Sized),
    account_id: &str,
) -> Result<Option<String>, TelegramError> {
    let credential_reader =
        ProviderCredentialReader::new(binding_store.clone(), secret_store.clone(), secret_resolver);
    match credential_reader
        .read(account_id, ProviderAccountSecretPurpose::TelegramSessionKey)
        .await
    {
        Ok(credential) => Ok(Some(credential.secret.expose_for_runtime().to_owned())),
        Err(ProviderCredentialError::MissingBinding { .. }) => Ok(None),
        Err(error) => Err(TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: {error}"
        ))),
    }
}
