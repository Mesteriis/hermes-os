use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;

use super::errors::EmailSyncPipelineError;
use super::organizations::project_email_participant_organization;
use super::participants::{EmailParticipant, parse_email_participant, upsert_message_participant};
use super::relationships::insert_relationship_event;

#[derive(Default)]
pub(crate) struct MessageKnowledgeReport {
    pub(crate) upserted_personas: usize,
    pub(crate) upserted_persona_identities: usize,
    pub(crate) upserted_message_participants: usize,
    pub(crate) upserted_relationship_events: usize,
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_persona_links: usize,
}

pub(crate) async fn project_message_knowledge(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<MessageKnowledgeReport, EmailSyncPipelineError> {
    let mut report = MessageKnowledgeReport::default();

    for message in messages {
        let mut participants = Vec::new();
        participants.push(parse_email_participant(&message.sender, "sender")?);
        for recipient in &message.recipients {
            participants.push(parse_email_participant(recipient, "recipient")?);
        }

        for participant in participants {
            let Some(persona_id) = confirmed_person_id_for_participant(pool, &participant).await?
            else {
                continue;
            };
            if upsert_message_participant(pool, message, &persona_id, &participant).await? {
                report.upserted_message_participants += 1;
            }
            if insert_relationship_event(pool, message, &persona_id, &participant).await? {
                report.upserted_relationship_events += 1;
            }

            let organization_report =
                project_email_participant_organization(pool, &persona_id, message, &participant)
                    .await?;
            report.upserted_organizations += organization_report.upserted_organizations;
            report.upserted_organization_persona_links +=
                organization_report.upserted_organization_persona_links;
        }
    }

    Ok(report)
}

async fn confirmed_person_id_for_participant(
    pool: &PgPool,
    participant: &EmailParticipant,
) -> Result<Option<String>, EmailSyncPipelineError> {
    let persona_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT persona_id
        FROM persona_identities
        WHERE identity_type = 'email'
          AND lower(identity_value) = $1
          AND status = 'active'
          AND persona_id IS NOT NULL
          AND source <> 'email_sync'
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&participant.email_address)
    .fetch_optional(pool)
    .await?;

    Ok(persona_id)
}
