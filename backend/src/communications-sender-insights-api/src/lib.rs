//! Generated public contract for provider-neutral Communications sender insights.

pub const PACKAGE: &str = "hermes-communications-sender-insights-api";
pub const SENDER_INSIGHTS_CONTRACT_NAME_V1: &str = "communications.sender-insights";
pub const SENDER_INSIGHTS_CONNECT_PATH_V1: &str =
    "/hermes.communications.sender_insights.v1.CommunicationsSenderInsightsService/List";
pub const SENDER_INSIGHTS_CONTRACT_MAJOR_V1: u32 = 1;
pub const SENDER_INSIGHTS_CONTRACT_REVISION_V1: u32 = 1;

mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.communications.sender_insights.v1.rs"
    ));
}

pub use wire::*;

include!(concat!(
    env!("OUT_DIR"),
    "/communications_sender_insights_schema.rs"
));

pub const COMMUNICATIONS_SENDER_INSIGHTS_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-sender-insights-v1.bin"
));
