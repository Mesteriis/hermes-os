ALTER TABLE graph_edges DROP CONSTRAINT IF EXISTS graph_edges_relationship_type;
ALTER TABLE graph_edges
ADD CONSTRAINT graph_edges_relationship_type CHECK (
    relationship_type IN (
        'person_has_email_address',
        'person_sent_message',
        'person_received_message',
        'email_address_sent_message',
        'email_address_received_message',
        'project_has_message',
        'project_has_document',
        'project_involves_person',
        'project_involves_email_address',
        'entity_relationship'
    )
);

ALTER TABLE graph_evidence DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;
ALTER TABLE graph_evidence
ADD CONSTRAINT graph_evidence_source_kind CHECK (
    source_kind IN ('contact', 'person', 'message', 'document', 'raw_record', 'relationship')
);
