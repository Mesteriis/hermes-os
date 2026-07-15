pub mod ids;

pub fn manifest() -> hermes_provider_api::ProviderManifest {
    hermes_provider_api::ProviderManifest::new(
        hermes_provider_api::ProviderId::parse("whatsapp").expect("static provider id"),
        1,
        ["webview.session", "messages.read", "messages.write"],
        [hermes_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static WhatsApp manifest")
}
