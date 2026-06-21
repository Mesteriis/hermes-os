-- Keep email finance schema aligned with the mail finance API model.

ALTER TABLE email_invoices
    ADD COLUMN IF NOT EXISTS linked_person_id TEXT;

CREATE INDEX IF NOT EXISTS email_invoices_linked_person_idx
    ON email_invoices (linked_person_id);
