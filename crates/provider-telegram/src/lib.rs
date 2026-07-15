pub mod tdlib;

pub fn manifest() -> hermes_provider_api::ProviderManifest {
    hermes_provider_api::ProviderManifest::new(
        hermes_provider_api::ProviderId::parse("telegram").expect("static provider id"),
        1,
        ["messages.read", "messages.write", "attachments.read"],
        [hermes_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Telegram manifest")
}
