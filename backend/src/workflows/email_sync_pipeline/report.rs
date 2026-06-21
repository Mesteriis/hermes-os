use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSyncPipelineReport {
    pub imported_records: usize,
    pub raw_blobs_upserted: usize,
    pub projected_messages: usize,
    pub attachment_blobs_upserted: usize,
    pub attachments_extracted: usize,
    pub attachments_not_scanned: usize,
    pub upserted_persons: usize,
    pub upserted_person_identities: usize,
    pub upserted_message_participants: usize,
    pub upserted_relationship_events: usize,
    pub upserted_organizations: usize,
    pub upserted_organization_contact_links: usize,
    pub refreshed_decision_candidates: usize,
    pub refreshed_knowledge_candidates: usize,
    pub refreshed_task_candidates: usize,
    pub checkpoint_saved: bool,
}
