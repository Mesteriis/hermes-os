use super::super::*;

#[derive(Serialize)]
pub(crate) struct PinToggleResponse {
    pub(in crate::app::handlers::communications) message_id: String,
    pub(in crate::app::handlers::communications) pinned: bool,
}

#[derive(Serialize)]
pub(crate) struct ImportantToggleResponse {
    pub(in crate::app::handlers::communications) message_id: String,
    pub(in crate::app::handlers::communications) important: bool,
}
