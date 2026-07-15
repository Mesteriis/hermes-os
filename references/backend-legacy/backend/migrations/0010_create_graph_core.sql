CREATE TABLE IF NOT EXISTS graph_nodes (
    node_id TEXT PRIMARY KEY,
    node_kind TEXT NOT NULL,
    stable_key TEXT NOT NULL,
    label TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_nodes_kind CHECK (node_kind IN ('person', 'email_address', 'message', 'document')),
    CONSTRAINT graph_nodes_stable_key_not_empty CHECK (length(trim(stable_key)) > 0),
    CONSTRAINT graph_nodes_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT graph_nodes_properties_is_object CHECK (jsonb_typeof(properties) = 'object'),
    UNIQUE (node_kind, stable_key)
);

CREATE TABLE IF NOT EXISTS graph_edges (
    edge_id TEXT PRIMARY KEY,
    source_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    target_node_id TEXT NOT NULL REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    confidence NUMERIC(5,4) NOT NULL,
    review_state TEXT NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_edges_relationship_type CHECK (
        relationship_type IN (
            'person_has_email_address',
            'person_sent_message',
            'person_received_message',
            'email_address_sent_message',
            'email_address_received_message'
        )
    ),
    CONSTRAINT graph_edges_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT graph_edges_review_state CHECK (
        review_state IN ('system_accepted', 'suggested', 'user_confirmed', 'user_rejected')
    ),
    CONSTRAINT graph_edges_properties_is_object CHECK (jsonb_typeof(properties) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS graph_edges_active_unique
ON graph_edges (source_node_id, target_node_id, relationship_type)
WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS graph_evidence (
    evidence_id TEXT PRIMARY KEY,
    edge_id TEXT NOT NULL REFERENCES graph_edges(edge_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    excerpt TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT graph_evidence_source_kind CHECK (source_kind IN ('contact', 'message', 'document', 'raw_record')),
    CONSTRAINT graph_evidence_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT graph_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (edge_id, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS graph_nodes_label_idx ON graph_nodes (label);
CREATE INDEX IF NOT EXISTS graph_edges_source_idx ON graph_edges (source_node_id);
CREATE INDEX IF NOT EXISTS graph_edges_target_idx ON graph_edges (target_node_id);
CREATE INDEX IF NOT EXISTS graph_evidence_edge_idx ON graph_evidence (edge_id);
