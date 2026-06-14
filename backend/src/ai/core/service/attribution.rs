use crate::domains::persons::api::PersonProjectionStore;

use super::super::agents::ai_agent_display_name;
use super::super::errors::AiError;
use super::core::AiService;

pub(super) struct AiRunAttribution {
    pub(super) agent_persona_id: String,
    pub(super) owner_persona_id: Option<String>,
}

impl AiService {
    pub(super) async fn run_attribution(
        &self,
        agent_id: &str,
    ) -> Result<AiRunAttribution, AiError> {
        let person_store = PersonProjectionStore::new(self.pool.clone());
        let agent_persona = person_store
            .upsert_ai_agent_persona(agent_id, ai_agent_display_name(agent_id)?)
            .await?;
        let owner_persona_id = person_store
            .owner_persona()
            .await?
            .map(|owner| owner.person_id);

        Ok(AiRunAttribution {
            agent_persona_id: agent_persona.person_id,
            owner_persona_id,
        })
    }
}
