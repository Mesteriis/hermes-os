#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    MailAddressBookEnvelopeBuildErrorV1, MailAddressBookEnvelopeContextV1,
    build_fetch_mail_address_book_page_command_v1, build_upsert_mail_address_book_entry_command_v1,
};

use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "hermes-mail-address-book-contract";
pub const MAIL_OWNER_ID_V1: &str = "mail";
pub const MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1: &str = "mail.address-book.provider.v1";
pub const MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1: u32 = 2;
pub const MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1: u32 = 500;
pub const MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1: usize = 4096;
pub const MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1: usize = 4096;
pub const MAIL_ADDRESS_BOOK_MAX_IN_FLIGHT_V1: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookContractV1 {
    FetchPageCommand,
    EntryObserved,
    PageCompleted,
    PageRejected,
    UpsertEntryCommand,
    EntryUpserted,
    EntryUpsertRejected,
}

impl MailAddressBookContractV1 {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FetchPageCommand => "mail_address_book_fetch_page",
            Self::EntryObserved => "mail_address_book_entry_observed",
            Self::PageCompleted => "mail_address_book_page_completed",
            Self::PageRejected => "mail_address_book_page_rejected",
            Self::UpsertEntryCommand => "mail_address_book_upsert_entry",
            Self::EntryUpserted => "mail_address_book_entry_upserted",
            Self::EntryUpsertRejected => "mail_address_book_entry_upsert_rejected",
        }
    }

    #[must_use]
    pub const fn envelope_kind(self) -> DurableEnvelopeKindV1 {
        match self {
            Self::FetchPageCommand | Self::UpsertEntryCommand => DurableEnvelopeKindV1::Command,
            Self::EntryObserved => DurableEnvelopeKindV1::Observation,
            Self::PageCompleted
            | Self::PageRejected
            | Self::EntryUpserted
            | Self::EntryUpsertRejected => DurableEnvelopeKindV1::Result,
        }
    }

    #[must_use]
    pub fn reference(self) -> ContractReferenceV1 {
        ContractReferenceV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: self.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }
    }

    #[must_use]
    pub fn publish_request(self) -> CapabilityRequestV1 {
        event_request(self, EventRouteDirectionV1::Publish)
    }

    #[must_use]
    pub fn consume_request(self) -> CapabilityRequestV1 {
        event_request(self, EventRouteDirectionV1::Consume)
    }
}

fn event_request(
    contract: MailAddressBookContractV1,
    direction: EventRouteDirectionV1,
) -> CapabilityRequestV1 {
    let consumes = direction == EventRouteDirectionV1::Consume;
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: contract.envelope_kind() as i32,
            contract: Some(contract.reference()),
            direction: direction as i32,
            max_in_flight: MAIL_ADDRESS_BOOK_MAX_IN_FLIGHT_V1,
            subscription_requirement: if consumes {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if consumes { 10 } else { 0 },
            ack_wait_millis: if consumes { 30_000 } else { 0 },
        })),
    }
}

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.mail.address_book.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_address_book_schema.rs"));

pub const MAIL_ADDRESS_BOOK_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-address-book-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_keeps_provider_protocol_in_mail_and_payloads_bounded() {
        let source = include_str!("../proto/hermes/mail/address_book/v1/address_book.proto");
        let fetch_command = message_source(source, "FetchMailAddressBookPageCommandV1");
        let observed_entry = message_source(source, "MailAddressBookEntryObservedV1");
        let upsert_command = message_source(source, "UpsertMailAddressBookEntryCommandV1");
        assert!(source.contains("GOOGLE_PEOPLE"));
        assert!(source.contains("ICLOUD_CARDDAV"));
        assert!(!upsert_command.contains("provider_kind"));
        assert!(!upsert_command.contains("provider_entry_id"));
        assert!(!upsert_command.contains("provider_etag"));
        assert!(source.contains("outcome_unknown"));
        assert!(!fetch_command.contains("provider_kind"));
        assert!(observed_entry.contains("provider_kind"));
        for forbidden in [
            "password",
            "access_token",
            "refresh_token",
            "cookie",
            "map<",
            "raw_json",
            "raw_xml",
        ] {
            assert!(!source.contains(forbidden), "forbidden field: {forbidden}");
        }
    }

    fn message_source<'a>(source: &'a str, name: &str) -> &'a str {
        let start = source
            .find(&format!("message {name} {{"))
            .expect("message start");
        let tail = &source[start..];
        let end = tail.find("\n}").expect("message end") + 2;
        &tail[..end]
    }

    #[test]
    fn descriptor_and_limits_are_non_empty() {
        assert!(!MAIL_ADDRESS_BOOK_DESCRIPTOR_SET_V1.is_empty());
        assert_ne!(MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1, [0; 32]);
        assert_eq!(MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, 500);
    }

    #[test]
    fn event_contracts_have_exact_mail_owner_kind_and_complementary_routes() {
        use hermes_runtime_protocol::v1::{
            DurableEnvelopeKindV1, EventRouteDirectionV1, capability_request_v1::Request,
        };

        let contracts = [
            MailAddressBookContractV1::FetchPageCommand,
            MailAddressBookContractV1::EntryObserved,
            MailAddressBookContractV1::PageCompleted,
            MailAddressBookContractV1::PageRejected,
            MailAddressBookContractV1::UpsertEntryCommand,
            MailAddressBookContractV1::EntryUpserted,
            MailAddressBookContractV1::EntryUpsertRejected,
        ];
        assert_eq!(
            contracts.map(MailAddressBookContractV1::name).as_slice(),
            [
                "mail_address_book_fetch_page",
                "mail_address_book_entry_observed",
                "mail_address_book_page_completed",
                "mail_address_book_page_rejected",
                "mail_address_book_upsert_entry",
                "mail_address_book_entry_upserted",
                "mail_address_book_entry_upsert_rejected",
            ]
        );
        assert_eq!(
            MailAddressBookContractV1::EntryObserved.envelope_kind(),
            DurableEnvelopeKindV1::Observation
        );
        for contract in contracts {
            assert_eq!(contract.reference().owner, MAIL_OWNER_ID_V1);
            let Some(Request::EventRoute(publish)) = contract.publish_request().request else {
                panic!("publish route");
            };
            let Some(Request::EventRoute(consume)) = contract.consume_request().request else {
                panic!("consume route");
            };
            assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
            assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
            assert_eq!(publish.contract, consume.contract);
        }
    }
}
