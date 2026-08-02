#![forbid(unsafe_code)]

pub const PACKAGE: &str = "hermes-mail-contacts-sync-api";
pub const MAIL_CONTACTS_SYNC_OWNER_ID_V1: &str = "mail_contacts_sync";
pub const MAIL_CONTACTS_SYNC_MODULE_ID_V1: &str = "hermes-mail-contacts-sync-runtime";
pub const MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1: &str = "mail.contacts-sync.v1";
pub const MAIL_CONTACTS_SYNC_COMMAND_CONNECT_PATH_V1: &str =
    "/hermes.mail_contacts_sync.v1.MailContactsSyncCommandService/Start";
pub const MAIL_CONTACTS_SYNC_QUERY_CONNECT_PATH_V1: &str =
    "/hermes.mail_contacts_sync.v1.MailContactsSyncQueryService/Get";
pub const MAIL_CONTACTS_SYNC_REALTIME_EVENT_KIND_V1: &str = "mail.contacts-sync.status-changed.v1";
pub const MAIL_CONTACTS_SYNC_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_CONTACTS_SYNC_CONTRACT_REVISION_V1: u32 = 1;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.mail_contacts_sync.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_contacts_sync_schema.rs"));

pub const MAIL_CONTACTS_SYNC_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-contacts-sync-v1.bin"));

#[cfg(test)]
mod tests {
    #[test]
    fn client_surface_is_generated_start_get_and_realtime_without_polling_contract() {
        let source = include_str!("../proto/hermes/mail_contacts_sync/v1/sync.proto");
        assert!(source.contains("rpc Start"));
        assert!(source.contains("rpc Get"));
        assert!(source.contains("MailContactsSyncStatusChangedV1"));
        for forbidden in [
            "Poll",
            "provider_entry_id",
            "provider_etag",
            "credential",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client surface: {forbidden}"
            );
        }
    }
}
