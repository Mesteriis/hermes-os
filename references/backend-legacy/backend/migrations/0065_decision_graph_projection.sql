ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN (
        'person',
        'email_address',
        'message',
        'document',
        'project',
        'decision'
    )
);

ALTER TABLE graph_evidence DROP CONSTRAINT IF EXISTS graph_evidence_source_kind;
ALTER TABLE graph_evidence
ADD CONSTRAINT graph_evidence_source_kind CHECK (
    source_kind IN (
        'contact',
        'person',
        'message',
        'document',
        'raw_record',
        'relationship',
        'decision'
    )
);
