INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_ai_semantic_embedding_v1',
    'AI_SEMANTIC_EMBEDDING',
    'AI Semantic Embedding',
    1,
    'ai',
    'Canonical evidence for derived semantic embedding materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
