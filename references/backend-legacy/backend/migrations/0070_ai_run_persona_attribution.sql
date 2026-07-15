ALTER TABLE ai_agent_runs
    ADD COLUMN IF NOT EXISTS agent_persona_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS owner_persona_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS ai_agent_runs_agent_persona_idx
    ON ai_agent_runs (agent_persona_id, started_at DESC)
    WHERE agent_persona_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS ai_agent_runs_owner_persona_idx
    ON ai_agent_runs (owner_persona_id, started_at DESC)
    WHERE owner_persona_id IS NOT NULL;
