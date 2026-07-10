-- Graph Persona naming alignment for node kinds, node ids and relationship types.

ALTER TABLE graph_evidence
    DROP CONSTRAINT IF EXISTS graph_evidence_edge_id_fkey,
    DROP CONSTRAINT IF EXISTS graph_evidence_edge_id_source_kind_source_id_key;

DROP INDEX IF EXISTS graph_edges_active_unique;

ALTER TABLE graph_edges
    DROP CONSTRAINT IF EXISTS graph_edges_source_node_id_fkey,
    DROP CONSTRAINT IF EXISTS graph_edges_target_node_id_fkey,
    DROP CONSTRAINT IF EXISTS graph_edges_relationship_type;

ALTER TABLE graph_nodes
    DROP CONSTRAINT IF EXISTS graph_nodes_kind;

CREATE TEMP TABLE graph_persona_node_renames AS
SELECT
    node_id AS old_node_id,
    format('graph:node:v1:persona:%s', stable_key) AS new_node_id,
    stable_key,
    ctid AS row_ctid,
    row_number() OVER (
        PARTITION BY stable_key
        ORDER BY
            CASE node_kind WHEN 'persona' THEN 0 ELSE 1 END,
            updated_at DESC,
            created_at DESC
    ) AS rank
FROM graph_nodes
WHERE node_kind IN ('person', 'persona');

UPDATE graph_edges edge
SET source_node_id = renames.new_node_id
FROM graph_persona_node_renames renames
WHERE edge.source_node_id = renames.old_node_id;

UPDATE graph_edges edge
SET target_node_id = renames.new_node_id
FROM graph_persona_node_renames renames
WHERE edge.target_node_id = renames.old_node_id;

DELETE FROM graph_nodes node
USING graph_persona_node_renames renames
WHERE node.ctid = renames.row_ctid
  AND renames.rank > 1;

UPDATE graph_nodes node
SET
    node_id = renames.new_node_id,
    node_kind = 'persona'
FROM graph_persona_node_renames renames
WHERE node.ctid = renames.row_ctid
  AND renames.rank = 1;

UPDATE graph_edges
SET relationship_type = CASE relationship_type
    WHEN 'person_has_email_address' THEN 'persona_has_email_address'
    WHEN 'person_sent_message' THEN 'persona_sent_message'
    WHEN 'person_received_message' THEN 'persona_received_message'
    WHEN 'project_involves_person' THEN 'project_involves_persona'
    ELSE relationship_type
END
WHERE relationship_type IN (
    'person_has_email_address',
    'person_sent_message',
    'person_received_message',
    'project_involves_person'
);

CREATE TEMP TABLE graph_persona_edge_renames AS
SELECT
    edge_id AS old_edge_id,
    format(
        'graph:edge:v1:%s:%s:%s:%s:%s:%s',
        length(source_node_id),
        source_node_id,
        length(relationship_type),
        relationship_type,
        length(target_node_id),
        target_node_id
    ) AS new_edge_id,
    ctid AS row_ctid,
    row_number() OVER (
        PARTITION BY
            source_node_id,
            target_node_id,
            relationship_type,
            valid_to IS NULL
        ORDER BY
            CASE WHEN valid_to IS NULL THEN 0 ELSE 1 END,
            updated_at DESC,
            created_at DESC
    ) AS rank
FROM graph_edges;

UPDATE graph_evidence evidence
SET edge_id = renames.new_edge_id
FROM graph_persona_edge_renames renames
WHERE evidence.edge_id = renames.old_edge_id;

DELETE FROM graph_edges edge
USING graph_persona_edge_renames renames
WHERE edge.ctid = renames.row_ctid
  AND renames.rank > 1;

UPDATE graph_edges edge
SET edge_id = renames.new_edge_id
FROM graph_persona_edge_renames renames
WHERE edge.ctid = renames.row_ctid
  AND renames.rank = 1;

CREATE TEMP TABLE graph_persona_evidence_renames AS
SELECT
    evidence_id AS old_evidence_id,
    format(
        'graph:evidence:v1:%s:%s:%s:%s:%s:%s',
        length(edge_id),
        edge_id,
        length(source_kind),
        source_kind,
        length(source_id),
        source_id
    ) AS new_evidence_id,
    ctid AS row_ctid,
    row_number() OVER (
        PARTITION BY edge_id, source_kind, source_id
        ORDER BY created_at DESC
    ) AS rank
FROM graph_evidence;

DELETE FROM graph_evidence evidence
USING graph_persona_evidence_renames renames
WHERE evidence.ctid = renames.row_ctid
  AND renames.rank > 1;

UPDATE graph_evidence evidence
SET evidence_id = renames.new_evidence_id
FROM graph_persona_evidence_renames renames
WHERE evidence.ctid = renames.row_ctid
  AND renames.rank = 1;

ALTER TABLE graph_nodes
    ADD CONSTRAINT graph_nodes_kind CHECK (
        node_kind IN (
            'persona',
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

ALTER TABLE graph_edges
    ADD CONSTRAINT graph_edges_relationship_type CHECK (
        relationship_type IN (
            'persona_has_email_address',
            'persona_sent_message',
            'persona_received_message',
            'email_address_sent_message',
            'email_address_received_message',
            'project_has_message',
            'project_has_document',
            'project_involves_persona',
            'project_involves_email_address',
            'entity_relationship'
        )
    ),
    ADD CONSTRAINT graph_edges_source_node_id_fkey
        FOREIGN KEY (source_node_id) REFERENCES graph_nodes(node_id) ON DELETE CASCADE,
    ADD CONSTRAINT graph_edges_target_node_id_fkey
        FOREIGN KEY (target_node_id) REFERENCES graph_nodes(node_id) ON DELETE CASCADE;

CREATE UNIQUE INDEX IF NOT EXISTS graph_edges_active_unique
ON graph_edges (source_node_id, target_node_id, relationship_type)
WHERE valid_to IS NULL;

ALTER TABLE graph_evidence
    ADD CONSTRAINT graph_evidence_edge_id_source_kind_source_id_key
        UNIQUE (edge_id, source_kind, source_id),
    ADD CONSTRAINT graph_evidence_edge_id_fkey
        FOREIGN KEY (edge_id) REFERENCES graph_edges(edge_id) ON DELETE CASCADE;
