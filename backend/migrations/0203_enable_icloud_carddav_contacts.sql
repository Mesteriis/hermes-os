UPDATE communication_provider_accounts
SET
    config = jsonb_set(
        config,
        '{connected_services}',
        COALESCE(config->'connected_services', '[]'::jsonb) || jsonb_build_array('contacts'),
        true
    ),
    updated_at = now()
WHERE provider_kind = 'icloud'
  AND COALESCE(config->>'auth_state', '') <> 'deleted'
  AND NOT (config ? 'deleted_at')
  AND NOT (COALESCE(config->'connected_services', '[]'::jsonb) ? 'contacts');
