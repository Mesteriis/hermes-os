-- Align provider address-book link identifiers with address-book terminology.

DO $$
BEGIN
    IF to_regclass('public.communication_provider_address_book_links') IS NOT NULL THEN
        IF EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'communication_provider_address_book_links'
              AND column_name = 'provider_contact_id'
        )
        AND NOT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'communication_provider_address_book_links'
              AND column_name = 'provider_address_book_entry_id'
        ) THEN
            ALTER TABLE communication_provider_address_book_links
                RENAME COLUMN provider_contact_id TO provider_address_book_entry_id;
        END IF;

        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'communication_provider_address_book_links_provider_contact_not_empty'
              AND conrelid = 'public.communication_provider_address_book_links'::regclass
        ) THEN
            ALTER TABLE communication_provider_address_book_links
                RENAME CONSTRAINT communication_provider_address_book_links_provider_contact_not_empty
                TO communication_provider_address_book_links_provider_address_book_entry_not_empty;
        END IF;
    END IF;
END $$;
