pub const MAIL_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/hermes.mail.v1.bin"));
pub const MAIL_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const MAIL_CLIENT_CONTRACT_REVISION: u32 = 5;
pub const MAIL_MODULE_ID: &str = "hermes-mail-runtime";
pub const MAIL_OWNER_ID: &str = "mail";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientContractV1 {
    AccountCredentialBind,
    AccountQuery,
    AccountRetire,
    AccountDelete,
    AccountLifecycleRetry,
    AccountLifecycleQuery,
    Sync,
    Delivery,
    DeliveryQuery,
    GmailOAuthStart,
    GmailOAuthComplete,
    GmailOAuthRefresh,
    GmailOAuthQuery,
}

impl MailClientContractV1 {
    pub const ALL: [Self; 13] = [
        Self::AccountCredentialBind,
        Self::AccountDelete,
        Self::AccountLifecycleQuery,
        Self::AccountLifecycleRetry,
        Self::AccountQuery,
        Self::AccountRetire,
        Self::Delivery,
        Self::DeliveryQuery,
        Self::GmailOAuthComplete,
        Self::GmailOAuthQuery,
        Self::GmailOAuthRefresh,
        Self::GmailOAuthStart,
        Self::Sync,
    ];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::AccountCredentialBind => "mail.account.credential.bind.v1",
            Self::AccountQuery => "mail.account.query.v1",
            Self::AccountRetire => "mail.account.retire.v1",
            Self::AccountDelete => "mail.account.delete.v1",
            Self::AccountLifecycleRetry => "mail.account.lifecycle.retry.v1",
            Self::AccountLifecycleQuery => "mail.account.lifecycle.query.v1",
            Self::Sync => "mail.sync.v1",
            Self::Delivery => "mail.delivery.v1",
            Self::DeliveryQuery => "mail.delivery.query.v1",
            Self::GmailOAuthStart => "mail.oauth.start.v1",
            Self::GmailOAuthComplete => "mail.oauth.complete.v1",
            Self::GmailOAuthRefresh => "mail.oauth.refresh.v1",
            Self::GmailOAuthQuery => "mail.oauth.query.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::AccountCredentialBind => {
                "/hermes.mail.account.v1.MailAccountCredentialBindingService/Bind"
            }
            Self::AccountQuery => "/hermes.mail.account.v1.MailAccountQueryService/Get",
            Self::AccountRetire => {
                "/hermes.mail.account_lifecycle.v1.MailAccountRetireService/Retire"
            }
            Self::AccountDelete => {
                "/hermes.mail.account_lifecycle.v1.MailAccountDeleteService/Delete"
            }
            Self::AccountLifecycleRetry => {
                "/hermes.mail.account_lifecycle.v1.MailAccountLifecycleRetryService/Retry"
            }
            Self::AccountLifecycleQuery => {
                "/hermes.mail.account_lifecycle.v1.MailAccountLifecycleStatusService/Get"
            }
            Self::Sync => "/hermes.mail.v1.MailSyncService/Sync",
            Self::Delivery => "/hermes.mail.v1.MailDeliveryCommandService/Send",
            Self::DeliveryQuery => "/hermes.mail.v1.MailDeliveryQueryService/GetOperationStatus",
            Self::GmailOAuthStart => "/hermes.mail.v1.GmailOAuthStartService/Start",
            Self::GmailOAuthComplete => "/hermes.mail.v1.GmailOAuthCompleteService/Complete",
            Self::GmailOAuthRefresh => "/hermes.mail.v1.GmailOAuthRefreshService/Refresh",
            Self::GmailOAuthQuery => "/hermes.mail.v1.GmailOAuthQueryService/GetOperationStatus",
        }
    }

    #[must_use]
    pub fn from_contract_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|contract| contract.contract_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn client_contracts_have_unique_capabilities_names_and_routes() {
        assert!(!MAIL_CLIENT_DESCRIPTOR_SET_V1.is_empty());
        assert_eq!(
            MailClientContractV1::ALL
                .into_iter()
                .map(MailClientContractV1::capability_id)
                .collect::<BTreeSet<_>>()
                .len(),
            MailClientContractV1::ALL.len()
        );
        assert_eq!(
            MailClientContractV1::ALL
                .into_iter()
                .map(MailClientContractV1::connect_path)
                .collect::<BTreeSet<_>>()
                .len(),
            MailClientContractV1::ALL.len()
        );
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            MailClientContractV1::from_contract_name("mail.client"),
            None
        );
    }
}
