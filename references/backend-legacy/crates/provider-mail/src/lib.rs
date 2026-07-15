pub mod gmail;

pub fn manifest() -> hermes_provider_api::ProviderManifest {
    hermes_provider_api::ProviderManifest::new(
        hermes_provider_api::ProviderId::parse("mail").expect("static provider id"),
        1,
        ["mail.read", "mail.write", "attachments.read"],
        [
            hermes_provider_api::RuntimeTopology::InProcess,
            hermes_provider_api::RuntimeTopology::SharedConnector,
        ],
    )
    .expect("static Mail manifest")
}
