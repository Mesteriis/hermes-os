ALTER TABLE secret_references
    DROP CONSTRAINT secret_references_store_kind;

ALTER TABLE secret_references
    ADD CONSTRAINT secret_references_store_kind CHECK (
        store_kind IN (
            'os_keychain',
            'encrypted_vault',
            'database_encrypted_vault',
            'host_vault',
            'external_vault',
            'test_double'
        )
    );
