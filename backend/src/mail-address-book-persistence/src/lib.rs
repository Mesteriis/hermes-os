#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod custody;
mod delivery;
mod model;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::MailAddressBookPersistenceConformanceV1;
pub use custody::MailAddressBookSnapshotCustodyOutcomeV1;
pub use delivery::{
    MailAddressBookCommandInboxOutcomeV1, MailAddressBookDispatchOutcomeV1,
    MailAddressBookResultStoreOutcomeV1,
};
pub use model::{
    MailAddressBookTargetSnapshotReceiptV1, MailAddressBookUpsertAdmissionV1,
    PendingMailAddressBookUpsertV1,
};
pub use repository::{MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1};
pub use schema::{
    MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1, MAIL_ADDRESS_BOOK_SCHEMA_V1,
    MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1, MailAddressBookSchemaErrorV1,
    append_mail_address_book_storage_v1,
};

pub const PACKAGE: &str = "hermes-mail-address-book-persistence";
