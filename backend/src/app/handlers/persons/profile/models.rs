use super::super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonListResponse {
    pub(super) items: Vec<crate::domains::persons::enrichment::EnrichedPerson>,
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<PersonaReadModel>,
}

#[derive(Serialize)]
pub(crate) struct PersonaReadModel {
    persona_id: String,
    persona_type: crate::domains::persons::api::PersonaType,
    is_self: bool,
    identity: PersonaIdentityReadModel,
    communication: PersonaCommunicationReadModel,
    compatibility: PersonaCompatibilityReadModel,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityReadModel {
    display_name: String,
    email_address: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCommunicationReadModel {
    primary_email: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCompatibilityReadModel {
    legacy_person_id: String,
    legacy_route: &'static str,
}

pub(super) fn persona_read_model(person: Person) -> PersonaReadModel {
    PersonaReadModel {
        persona_id: person.person_id.clone(),
        persona_type: person.persona_type,
        is_self: person.is_self,
        identity: PersonaIdentityReadModel {
            display_name: person.display_name,
            email_address: person.email_address.clone(),
        },
        communication: PersonaCommunicationReadModel {
            primary_email: person.email_address,
        },
        compatibility: PersonaCompatibilityReadModel {
            legacy_person_id: person.person_id,
            legacy_route: "/api/v1/persons",
        },
        created_at: person.created_at,
        updated_at: person.updated_at,
    }
}
