use std::collections::BTreeSet;

use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::persons::api::PersonProjectionPort;

use super::errors::EmailSyncPipelineError;
use super::organizations::project_email_participant_organization;
use super::participants::{parse_email_participant, upsert_message_participant};
use super::relationships::insert_relationship_event;

#[derive(Default)]
pub(crate) struct MessageKnowledgeReport {
    pub(crate) upserted_persons: usize,
    pub(crate) upserted_person_identities: usize,
    pub(crate) upserted_message_participants: usize,
    pub(crate) upserted_relationship_events: usize,
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_contact_links: usize,
}

pub(crate) async fn project_message_knowledge(
    pool: &PgPool,
    person_store: &PersonProjectionPort,
    messages: &[ProjectedMessage],
) -> Result<MessageKnowledgeReport, EmailSyncPipelineError> {
    let mut report = MessageKnowledgeReport::default();
    let mut seen_persons = BTreeSet::new();

    for message in messages {
        let mut participants = Vec::new();
        participants.push(parse_email_participant(&message.sender, "sender")?);
        for recipient in &message.recipients {
            participants.push(parse_email_participant(recipient, "recipient")?);
        }

        for participant in participants {
            let person = person_store
                .upsert_email_person_with_observation(
                    &participant.email_address,
                    &message.observation_id,
                )
                .await?;
            if seen_persons.insert(person.person_id.clone()) {
                report.upserted_persons += 1;
                report.upserted_person_identities += 1;
            }
            if upsert_message_participant(pool, message, &person.person_id, &participant).await? {
                report.upserted_message_participants += 1;
            }
            if insert_relationship_event(pool, message, &person.person_id, &participant).await? {
                report.upserted_relationship_events += 1;
            }

            let organization_report = project_email_participant_organization(
                pool,
                &person.person_id,
                message,
                &participant,
            )
            .await?;
            report.upserted_organizations += organization_report.upserted_organizations;
            report.upserted_organization_contact_links +=
                organization_report.upserted_organization_contact_links;
        }
    }

    Ok(report)
}
