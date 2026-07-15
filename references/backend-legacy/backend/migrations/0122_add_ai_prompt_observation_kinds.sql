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
        'okd_ai_prompt_template_v1',
        'AI_PROMPT_TEMPLATE',
        'AI Prompt Template',
        1,
        'ai',
        'Canonical evidence for AI prompt template lifecycle mutations.'
    ),
    (
        'okd_ai_prompt_template_version_v1',
        'AI_PROMPT_TEMPLATE_VERSION',
        'AI Prompt Template Version',
        1,
        'ai',
        'Canonical evidence for AI prompt template version lifecycle mutations.'
    ),
    (
        'okd_ai_prompt_eval_run_v1',
        'AI_PROMPT_EVAL_RUN',
        'AI Prompt Eval Run',
        1,
        'ai',
        'Canonical evidence for AI prompt preview and evaluation runs.'
    )
ON CONFLICT (code, version) DO NOTHING;
