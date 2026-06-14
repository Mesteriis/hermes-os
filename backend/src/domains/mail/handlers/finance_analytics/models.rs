use super::super::*;

#[derive(Serialize)]
pub(crate) struct PinToggleResponse {
    pub(in crate::domains::mail::handlers) message_id: String,
    pub(in crate::domains::mail::handlers) pinned: bool,
}

#[derive(Serialize)]
pub(crate) struct ImportantToggleResponse {
    pub(in crate::domains::mail::handlers) message_id: String,
    pub(in crate::domains::mail::handlers) important: bool,
}
