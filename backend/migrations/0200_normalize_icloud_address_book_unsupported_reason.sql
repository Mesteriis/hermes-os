UPDATE communication_provider_accounts
SET
    config = config || jsonb_build_object(
        'address_book_sync_unsupported_reason',
        'icloud_address_book_adapter_not_configured'
    ),
    updated_at = now()
WHERE provider_kind = 'icloud'
  AND config->>'address_book_sync_unsupported_reason' = 'icloud_contacts_adapter_not_configured';
