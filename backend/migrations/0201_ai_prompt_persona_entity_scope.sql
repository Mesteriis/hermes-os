UPDATE ai_prompt_templates
SET entity_scope = 'persona',
    updated_at = now()
WHERE entity_scope = 'person';

ALTER TABLE ai_prompt_templates
    DROP CONSTRAINT IF EXISTS ai_prompt_templates_entity_scope_check;

ALTER TABLE ai_prompt_templates
    ADD CONSTRAINT ai_prompt_templates_entity_scope_check CHECK (
        entity_scope IN (
            'global',
            'persona',
            'organization',
            'project',
            'document',
            'task',
            'meeting',
            'communication',
            'conversation'
        )
    );
