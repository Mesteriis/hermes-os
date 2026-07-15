-- Phase 0: Rename contacts -> persons
-- Renames tables and updates ID values from contact:v1: to person:v1:

ALTER TABLE contacts RENAME TO persons;
ALTER TABLE contact_identity_candidates RENAME TO person_identity_candidates;

-- Rename columns in the renamed tables
ALTER TABLE persons RENAME COLUMN contact_id TO person_id;
ALTER TABLE persons RENAME COLUMN contact_metadata TO person_metadata;

ALTER TABLE person_identity_candidates RENAME COLUMN left_contact_id TO left_person_id;
ALTER TABLE person_identity_candidates RENAME COLUMN right_contact_id TO right_person_id;

-- Update ID values: contact:v1: -> person:v1:
UPDATE persons SET person_id = replace(person_id, 'contact:v1:', 'person:v1:');
UPDATE person_identity_candidates SET left_person_id = replace(left_person_id, 'contact:v1:', 'person:v1:');
UPDATE person_identity_candidates SET right_person_id = replace(right_person_id, 'contact:v1:', 'person:v1:');

-- Update event_log payloads
UPDATE event_log SET event_id = replace(event_id, 'contact_identity_review:', 'person_identity_review:') WHERE event_id LIKE 'contact_identity_review:%';
UPDATE event_log SET event_type = replace(event_type, 'contact_identity.', 'person_identity.') WHERE event_type LIKE 'contact_identity.%';

-- Update graph nodes
UPDATE graph_nodes SET node_id = replace(node_id, 'contact:v1:', 'person:v1:') WHERE node_id LIKE 'contact:v1:%';

-- Rename constraints
ALTER TABLE persons RENAME CONSTRAINT contacts_display_name_not_empty TO persons_display_name_not_empty;
ALTER TABLE persons RENAME CONSTRAINT contacts_email_not_empty TO persons_email_not_empty;
ALTER TABLE persons RENAME CONSTRAINT contacts_pkey TO persons_pkey;
ALTER TABLE persons RENAME CONSTRAINT contacts_trust_score_range TO persons_trust_score_range;
ALTER TABLE persons RENAME CONSTRAINT contacts_contact_metadata_is_object TO persons_person_metadata_is_object;

-- Rename indexes
ALTER INDEX contacts_email_address_key RENAME TO persons_email_address_key;
ALTER INDEX contacts_trust_score_idx RENAME TO persons_trust_score_idx;
ALTER INDEX contacts_last_interaction_idx RENAME TO persons_last_interaction_idx;
ALTER INDEX contacts_favorite_idx RENAME TO persons_favorite_idx;
