-- Align Review item kind naming with the Persona domain.
--
-- The application keeps accepting legacy `new_person` rows at read boundaries,
-- but new writes and the database constraint should use `new_persona`.

UPDATE review_items
SET item_kind = 'new_persona'
WHERE item_kind = 'new_person';

ALTER TABLE review_items
    DROP CONSTRAINT IF EXISTS review_items_item_kind;

ALTER TABLE review_items
    ADD CONSTRAINT review_items_item_kind CHECK (
        item_kind IN (
            'new_persona',
            'new_organization',
            'identity_candidate',
            'project_link_candidate',
            'contradiction_candidate',
            'potential_task',
            'potential_obligation',
            'potential_decision',
            'potential_relationship',
            'potential_project',
            'knowledge_candidate'
        )
    );
