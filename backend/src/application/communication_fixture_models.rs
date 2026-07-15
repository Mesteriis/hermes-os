pub(crate) struct WhatsappParticipantUpsertOutcome {
    pub(crate) participant_id: String,
    pub(crate) previous_role: Option<String>,
    pub(crate) previous_status: Option<String>,
    pub(crate) role_changed: bool,
    pub(crate) membership_changed: bool,
}

pub(crate) struct AcceptedWhatsappRawRecord {
    pub(crate) raw_record_id: String,
    pub(crate) accepted_event_id: String,
    pub(crate) observation_id: String,
}

pub(crate) struct WhatsappAccountProjectionContext {
    pub(crate) provider_kind: String,
    pub(crate) channel_kind: String,
}
