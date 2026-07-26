pub const PACKAGE: &str = "hermes-telegram-call-media-contract";
pub const TD_CALL_MIN_LAYER_V1: i32 = 65;
pub const TD_CALL_MAX_LAYER_V1: i32 = 92;
pub const MAX_LIBRARY_VERSION_BYTES: usize = 128;
pub const MAX_LIBRARY_VERSIONS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallProtocolV1 {
    pub udp_p2p: bool,
    pub udp_reflector: bool,
    pub min_layer: i32,
    pub max_layer: i32,
    pub library_versions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallMediaContractError {
    InvalidProtocol,
    Unavailable,
    SessionNotFound,
    InvalidState,
}

impl TelegramCallProtocolV1 {
    pub fn new(
        udp_p2p: bool,
        udp_reflector: bool,
        library_versions: Vec<String>,
    ) -> Result<Self, TelegramCallMediaContractError> {
        let protocol = Self {
            udp_p2p,
            udp_reflector,
            min_layer: TD_CALL_MIN_LAYER_V1,
            max_layer: TD_CALL_MAX_LAYER_V1,
            library_versions,
        };
        protocol.validate()?;
        Ok(protocol)
    }

    pub fn validate(&self) -> Result<(), TelegramCallMediaContractError> {
        if (!self.udp_p2p && !self.udp_reflector)
            || self.min_layer != TD_CALL_MIN_LAYER_V1
            || self.max_layer != TD_CALL_MAX_LAYER_V1
            || self.library_versions.is_empty()
            || self.library_versions.len() > MAX_LIBRARY_VERSIONS
            || self.library_versions.iter().any(|version| {
                version.trim().is_empty()
                    || version.len() > MAX_LIBRARY_VERSION_BYTES
                    || version.chars().any(char::is_control)
            })
        {
            return Err(TelegramCallMediaContractError::InvalidProtocol);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelegramCallDiscardContextV1 {
    pub duration_seconds: u32,
    pub connection_id: i64,
}

pub trait TelegramCallSignalingMediaPort {
    fn supported_protocol(&self) -> Result<TelegramCallProtocolV1, TelegramCallMediaContractError>;

    fn discard_context(
        &self,
        call_session_id: &str,
    ) -> Result<TelegramCallDiscardContextV1, TelegramCallMediaContractError>;

    fn set_local_mute(
        &mut self,
        call_session_id: &str,
        muted: bool,
    ) -> Result<(), TelegramCallMediaContractError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_requires_exact_tdlib_layers_and_one_pinned_library_version() {
        let protocol = TelegramCallProtocolV1::new(true, true, vec!["pinned-tgcalls".to_owned()])
            .expect("protocol");

        assert_eq!(protocol.min_layer, 65);
        assert_eq!(protocol.max_layer, 92);
        assert_eq!(
            TelegramCallProtocolV1::new(true, true, Vec::new()),
            Err(TelegramCallMediaContractError::InvalidProtocol)
        );
    }
}
