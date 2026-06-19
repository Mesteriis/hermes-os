use sqlx::Row;

use super::errors::CommunicationIngestionError;
use super::models::{
    DeletedProviderAccount, NewProviderAccount, ProviderAccount, ProviderAccountUsage,
};
use super::store::CommunicationIngestionStore;
use serde_json::Value;

impl CommunicationIngestionStore {
    pub async fn upsert_provider_account(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .upsert(account)
            .await
    }

    pub async fn provider_account(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
            .await
    }

    pub async fn list_provider_accounts(
        &self,
    ) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .list()
            .await
    }

    pub async fn update_provider_account_config(
        &self,
        account_id: &str,
        config: &Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .update_config(account_id, config)
            .await
    }

    pub async fn provider_account_usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .usage(account_id)
            .await
    }

    pub async fn delete_provider_account_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        crate::vault::CommunicationProviderAccountStore::new(self.pool.clone())
            .delete_metadata(account_id)
            .await
    }
}
