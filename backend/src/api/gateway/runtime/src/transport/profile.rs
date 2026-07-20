use std::net::IpAddr;

/// The only externally reachable Gateway listener profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayTransportProfileV1 {
    LocalEmbedded,
    PairedRemote(PairedRemoteProfileV1),
}

/// Remote access is always TLS-protected and uses HTTP/2 as its baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairedRemoteProfileV1 {
    http3_enabled: bool,
    early_data_enabled: bool,
}

impl PairedRemoteProfileV1 {
    pub const fn new(http3_enabled: bool, early_data_enabled: bool) -> Result<Self, &'static str> {
        if early_data_enabled {
            return Err("Gateway HTTP/3 early data is forbidden");
        }
        Ok(Self {
            http3_enabled,
            early_data_enabled,
        })
    }

    #[must_use]
    pub const fn http3_enabled(self) -> bool {
        self.http3_enabled
    }

    #[must_use]
    pub const fn early_data_enabled(self) -> bool {
        self.early_data_enabled
    }
}

impl GatewayTransportProfileV1 {
    /// Rejects an accidentally remote plaintext listener before it is bound.
    pub fn validate_bind(self, address: IpAddr, tls_enabled: bool) -> Result<(), &'static str> {
        match self {
            Self::LocalEmbedded if address.is_loopback() => Ok(()),
            Self::LocalEmbedded => Err("local Gateway listener must bind loopback only"),
            Self::PairedRemote(_) if !tls_enabled => {
                Err("paired remote Gateway listener requires TLS")
            }
            Self::PairedRemote(_) => Ok(()),
        }
    }
}
