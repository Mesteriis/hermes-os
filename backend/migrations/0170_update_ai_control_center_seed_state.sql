UPDATE ai_model_catalog
SET is_available = false
WHERE provider_id = 'provider:built_in:ollama'
    AND metadata ->> 'pull_required' = 'true'
    AND is_available IS DISTINCT FROM false;

DELETE FROM ai_model_routes
WHERE provider_id = 'provider:built_in:ollama';
