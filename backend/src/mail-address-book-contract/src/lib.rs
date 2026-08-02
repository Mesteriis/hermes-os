#![forbid(unsafe_code)]

pub const PACKAGE: &str = "hermes-mail-address-book-contract";
pub const MAIL_OWNER_ID_V1: &str = "mail";
pub const MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1: &str = "mail.address-book.provider.v1";
pub const MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1: u32 = 1;
pub const MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1: u32 = 500;
pub const MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1: usize = 4096;
pub const MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1: usize = 4096;

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
        assert!(source.contains("GOOGLE_PEOPLE"));
        assert!(source.contains("ICLOUD_CARDDAV"));
        assert!(source.contains("expected_provider_etag"));
        assert!(source.contains("outcome_unknown"));
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

    #[test]
    fn descriptor_and_limits_are_non_empty() {
        assert!(!MAIL_ADDRESS_BOOK_DESCRIPTOR_SET_V1.is_empty());
        assert_ne!(MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1, [0; 32]);
        assert_eq!(MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, 500);
    }
}
