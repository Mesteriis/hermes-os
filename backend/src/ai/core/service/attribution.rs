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
        let persona_attribution = self
            .persona_attribution
            .as_ref()
            .ok_or(AiError::PersonaAttributionUnavailable)?;
        let agent_persona = persona_attribution
            .upsert_ai_agent_persona(agent_id, ai_agent_display_name(agent_id)?)
            .await?;
        let owner_persona_id = persona_attribution.owner_persona_id().await?;

        Ok(AiRunAttribution {
            agent_persona_id: agent_persona.persona_id,
            owner_persona_id,
        })
    }
}
