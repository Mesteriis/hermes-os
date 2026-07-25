pub const MAIL_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/hermes.mail.v1.bin"));
pub const MAIL_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const MAIL_CLIENT_CONTRACT_REVISION: u32 = 1;
pub const MAIL_MODULE_ID: &str = "hermes-mail-runtime";
pub const MAIL_OWNER_ID: &str = "mail";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientContractV1 {
    Sync,
    Delivery,
}

impl MailClientContractV1 {
    pub const ALL: [Self; 2] = [Self::Delivery, Self::Sync];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Sync => "mail.sync.v1",
            Self::Delivery => "mail.delivery.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Sync => "/hermes.mail.v1.MailSyncService/Sync",
            Self::Delivery => "/hermes.mail.v1.MailDeliveryService/Send",
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
