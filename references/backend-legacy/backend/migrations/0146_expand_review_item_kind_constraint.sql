ALTER TABLE review_items
    DROP CONSTRAINT IF EXISTS review_items_item_kind;

ALTER TABLE review_items
    ADD CONSTRAINT review_items_item_kind CHECK (
        item_kind IN (
            'new_person',
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
