import type { AiProviderPreset } from '../types/aiControlCenter'

export const DEFAULT_OPENAI_BASE_URL = 'https://api.openai.com/v1'

export const SLOT_LABELS: Record<string, string> = {
  default_chat: 'Translation and general chat',
  reasoning: 'Reasoning',
  summarization: 'Summaries',
  mail_intelligence: 'Mail analysis',
  reply_draft: 'Reply drafts',
  extraction: 'Extraction and categorization',
  embeddings: 'Embeddings',
  meeting_prep: 'Meeting preparation',
}

export const SLOT_DESCRIPTIONS: Record<string, string> = {
  default_chat: 'Default text generation, translation and short assistant actions.',
  reasoning: 'Deep reasoning and multi-step analysis.',
  summarization: 'Summaries for messages, documents and context packs.',
  mail_intelligence: 'Mail triage, sentiment, urgency and signal extraction.',
  reply_draft: 'Drafting replies for mail and messaging flows.',
  extraction: 'Entity extraction, classification and categorization.',
  embeddings: 'Semantic index embeddings. Dimension must match backend requirements.',
  meeting_prep: 'Meeting prep, agendas and follow-up intelligence.',
}

export const DEFAULT_API_PROVIDER_PRESETS: AiProviderPreset[] = [
  apiProviderPreset('raw', 'Raw OpenAI-compatible API', null, [
    'chat',
    'reasoning',
    'summarization',
    'embeddings',
    'extraction',
  ]),
  apiProviderPreset('openai', 'OpenAI', 'https://api.openai.com/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('deepseek', 'DeepSeek', 'https://api.deepseek.com/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('openrouter', 'OpenRouter', 'https://openrouter.ai/api/v1', [
    'chat',
    'reasoning',
    'routing',
  ]),
  apiProviderPreset('groq', 'Groq', 'https://api.groq.com/openai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('together', 'Together AI', 'https://api.together.xyz/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('fireworks', 'Fireworks AI', 'https://api.fireworks.ai/inference/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('mistral', 'Mistral AI', 'https://api.mistral.ai/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('xai', 'xAI', 'https://api.x.ai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset(
    'gemini-openai',
    'Google Gemini OpenAI-compatible',
    'https://generativelanguage.googleapis.com/v1beta/openai',
    ['chat', 'reasoning', 'embeddings']
  ),
  apiProviderPreset('perplexity', 'Perplexity', 'https://api.perplexity.ai', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('nvidia-nim', 'NVIDIA NIM', 'https://integrate.api.nvidia.com/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('cerebras', 'Cerebras', 'https://api.cerebras.ai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('lm-studio', 'LM Studio', 'http://127.0.0.1:1234/v1', [
    'chat',
    'local_runtime',
  ]),
  apiProviderPreset('vllm-local', 'vLLM local', 'http://127.0.0.1:8000/v1', [
    'chat',
    'local_runtime',
  ]),
  apiProviderPreset('omniroute', 'OmniRoute', 'https://ai.sh-inc.ru/v1', [
    'chat',
    'embeddings',
    'routing',
  ]),
]

function apiProviderPreset(
  providerKey: string,
  displayName: string,
  baseUrl: string | null,
  capabilities: string[]
): AiProviderPreset {
  return {
    provider_kind: 'api',
    provider_key: providerKey,
    display_name: displayName,
    privacy: 'remote',
    base_url: baseUrl,
    command_preset: null,
    capabilities,
  }
}
