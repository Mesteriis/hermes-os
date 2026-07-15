INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_ai_model_catalog_item_v1',
    'AI_MODEL_CATALOG_ITEM',
    'AI Model Catalog Item',
    1,
    'ai',
    'Canonical evidence for AI curated model catalog materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
