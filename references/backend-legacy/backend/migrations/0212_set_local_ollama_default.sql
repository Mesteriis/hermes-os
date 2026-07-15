-- Preserve explicitly configured endpoints while moving the project default to the local Ollama host.
UPDATE application_settings
SET
    value = '"http://192.168.1.2:11434"'::jsonb,
    metadata = jsonb_set(metadata, '{placeholder}', '"http://192.168.1.2:11434"'::jsonb, true),
    updated_at = now()
WHERE setting_key = 'ai.ollama_base_url'
  AND value = '"http://127.0.0.1:11434"'::jsonb;

UPDATE ai_provider_accounts
SET
    config = jsonb_set(config, '{base_url}', '"http://192.168.1.2:11434"'::jsonb, true),
    updated_at = now()
WHERE provider_kind = 'built_in'
  AND provider_key = 'ollama'
  AND config ->> 'base_url' = 'http://127.0.0.1:11434';
