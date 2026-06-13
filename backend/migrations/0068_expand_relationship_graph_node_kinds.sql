ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN (
        'person',
        'email_address',
        'message',
        'document',
        'project',
        'organization',
        'task',
        'event',
        'decision',
        'obligation',
        'knowledge'
    )
);
