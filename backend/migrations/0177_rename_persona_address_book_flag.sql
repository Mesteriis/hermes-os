DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'persons'
          AND column_name = 'is_contact'
    )
    AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'persons'
          AND column_name = 'is_address_book'
    ) THEN
        ALTER TABLE persons RENAME COLUMN is_contact TO is_address_book;
    END IF;
END $$;

ALTER INDEX IF EXISTS persons_is_contact_idx RENAME TO persons_is_address_book_idx;
