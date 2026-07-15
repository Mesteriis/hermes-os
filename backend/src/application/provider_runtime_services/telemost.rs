use super::*;

impl YandexTelemostProviderRuntimeApplicationService {
    pub(crate) fn new(store: YandexTelemostStore) -> Self {
        Self { store }
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<YandexTelemostAccountListResponse, YandexTelemostError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &YandexTelemostRetentionCleanupRequest,
    ) -> Result<YandexTelemostRetentionCleanupResponse, YandexTelemostError> {
        self.store.cleanup_retention(account_id, request).await
    }
}
