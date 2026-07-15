-- Graph tables are derived projection state per ADR-0045. Rebuilding them during
-- the V3 pgvector upgrade avoids carrying forward corrupted local projection rows
-- from earlier dev smoke runs while preserving canonical source records.
TRUNCATE TABLE graph_evidence, graph_edges, graph_nodes;
