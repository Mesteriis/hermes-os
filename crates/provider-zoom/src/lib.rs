pub mod protocol;

pub fn manifest() -> hermes_provider_api::ProviderManifest {
    hermes_provider_api::ProviderManifest::new(
        hermes_provider_api::ProviderId::parse("zoom").expect("static provider id"),
        1,
        ["meetings.read", "recordings.read"],
        [hermes_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Zoom manifest")
}
