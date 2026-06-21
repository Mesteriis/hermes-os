CREATE TABLE IF NOT EXISTS contacts (
    contact_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    email_address TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT contacts_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT contacts_email_not_empty CHECK (length(trim(email_address)) > 0)
);
