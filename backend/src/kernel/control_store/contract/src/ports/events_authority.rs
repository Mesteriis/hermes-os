use crate::PlatformEventsAuthorityConfigurationV1;

/// Kernel-owned desired configuration for the managed Events authority.
pub trait EventsAuthorityStore {
    type Error;

    fn record_platform_events_authority_configuration(
        &self,
        configuration: &PlatformEventsAuthorityConfigurationV1,
    ) -> Result<(), Self::Error>;

    fn platform_events_authority_configuration(
        &self,
    ) -> Result<Option<PlatformEventsAuthorityConfigurationV1>, Self::Error>;
}
