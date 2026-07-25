pub const ZULIP_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/hermes.zulip.v1.bin"));
pub const ZULIP_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const ZULIP_CLIENT_CONTRACT_REVISION: u32 = 1;
pub const ZULIP_MODULE_ID: &str = "hermes-zulip-runtime";
pub const ZULIP_OWNER_ID: &str = "zulip";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipClientContractV1 {
    Command,
    Query,
}

impl ZulipClientContractV1 {
    pub const ALL: [Self; 2] = [Self::Command, Self::Query];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Command => "zulip.command.v1",
            Self::Query => "zulip.query.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Command => "/hermes.zulip.v1.ZulipCommandService/ExecuteCommand",
            Self::Query => "/hermes.zulip.v1.ZulipQueryService/GetOperationStatus",
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
        assert!(!ZULIP_CLIENT_DESCRIPTOR_SET_V1.is_empty());
        assert_eq!(
            ZulipClientContractV1::ALL
                .into_iter()
                .map(ZulipClientContractV1::capability_id)
                .collect::<BTreeSet<_>>()
                .len(),
            ZulipClientContractV1::ALL.len()
        );
        assert_eq!(
            ZulipClientContractV1::ALL
                .into_iter()
                .map(ZulipClientContractV1::connect_path)
                .collect::<BTreeSet<_>>()
                .len(),
            ZulipClientContractV1::ALL.len()
        );
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            ZulipClientContractV1::from_contract_name("zulip.client"),
            None
        );
    }
}
