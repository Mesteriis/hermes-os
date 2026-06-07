use hermes_hub_backend::domains::persons::core::{
    NewPersonPersona, PersonPersonaStore, PersonsIdentityStore,
};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct ContactFactory<'a> {
    pool: &'a PgPool,
    display_name: String,
    email: Option<String>,
    person_id: Option<String>,
}

impl<'a> ContactFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            display_name: format!(
                "Test Person {}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
            email: None,
            person_id: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_person_id(mut self, id: impl Into<String>) -> Self {
        self.person_id = Some(id.into());
        self
    }

    /// Create a person identity and a default persona. Returns the person ID.
    pub async fn create(
        self,
    ) -> Result<String, hermes_hub_backend::domains::persons::core::PersonCoreError> {
        let identity_store = PersonsIdentityStore::new(self.pool.clone());
        let persona_store = PersonPersonaStore::new(self.pool.clone());

        let person_id = self
            .person_id
            .unwrap_or_else(|| format!("person:{}", Uuid::new_v4()));

        // Create identity via upsert
        identity_store
            .upsert(&person_id, "email", &self.display_name, "testkit")
            .await?;

        // Create a default persona
        let persona = NewPersonPersona {
            persona_id: format!("persona:{}", Uuid::new_v4()),
            person_id: person_id.clone(),
            name: self.display_name,
            context: Some("test".into()),
            default_tone: Some("neutral".into()),
            default_language: Some("en".into()),
            preferred_channel: self.email,
        };
        persona_store.upsert(&persona).await?;

        Ok(person_id)
    }
}
