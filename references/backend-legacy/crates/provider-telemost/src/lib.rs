pub mod models;
pub mod protocol;

pub fn manifest() -> hermes_provider_api::ProviderManifest {
    hermes_provider_api::ProviderManifest::new(
        hermes_provider_api::ProviderId::parse("telemost").expect("static provider id"),
        1,
        ["meetings.read", "meetings.write"],
        [hermes_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Telemost manifest")
}
