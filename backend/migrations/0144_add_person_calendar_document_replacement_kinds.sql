INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES
(
    'observation_kind:v1:person_memory_card',
    'PERSON_MEMORY_CARD',
    'Person memory card',
    1,
    'persons',
    'Canonical evidence describing a manual person memory note or memory card captured into persona memory.'
),
(
    'observation_kind:v1:event_agenda',
    'EVENT_AGENDA',
    'Event agenda',
    1,
    'calendar',
    'Canonical evidence describing a manual agenda captured for a calendar event.'
),
(
    'observation_kind:v1:event_checklist',
    'EVENT_CHECKLIST',
    'Event checklist',
    1,
    'calendar',
    'Canonical evidence describing a manual checklist captured for a calendar event.'
),
(
    'observation_kind:v1:meeting_note',
    'MEETING_NOTE',
    'Meeting note',
    1,
    'calendar',
    'Canonical evidence describing a manual meeting note captured for a calendar event.'
),
(
    'observation_kind:v1:calendar_rule',
    'CALENDAR_RULE',
    'Calendar rule',
    1,
    'calendar',
    'Canonical evidence describing a manual create, update, or delete mutation of a calendar rule.'
)
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
