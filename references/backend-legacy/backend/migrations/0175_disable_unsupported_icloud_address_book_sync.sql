UPDATE communication_provider_accounts
SET
    config = jsonb_set(
        config,
        '{connected_services}',
        (
            SELECT COALESCE(jsonb_agg(service), '[]'::jsonb)
            FROM jsonb_array_elements(config->'connected_services') AS service
            WHERE service <> to_jsonb('contacts'::text)
        ),
        true
    )
    || jsonb_build_object(
        'address_book_sync_enabled', false,
        'address_book_sync_unsupported_reason', 'icloud_address_book_adapter_not_configured'
    ),
    updated_at = now()
WHERE provider_kind = 'icloud'
  AND jsonb_typeof(config->'connected_services') = 'array'
  AND (config->'connected_services') ? 'contacts';
