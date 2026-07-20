use crate::PlatformEventHubTopologyV1;

/// Kernel-only desired Event Hub topology, excluding broker credentials and payloads.
pub trait EventHubTopologyStore {
    type Error;

    fn record_platform_event_hub_topology(
        &self,
        topology: &PlatformEventHubTopologyV1,
    ) -> Result<(), Self::Error>;

    fn platform_event_hub_topology(
        &self,
    ) -> Result<Option<PlatformEventHubTopologyV1>, Self::Error>;
}
