import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'
import { isRecord } from '../../../shared/communications/queries/realtimePatchShared'

type Translate = (key: string) => string

export interface AiModelCapabilityBadge {
  key: string
  label: string
  muted: boolean
}

export interface AiModelRuntimeFact {
  key: string
  label: string
  value: string
}

export function modelMatchesSearch(
  model: AiModelCatalogItem,
  provider: AiProviderAccount,
  query: string
): boolean {
  return modelSearchTokens(model, provider).some((token) => token.toLowerCase().includes(query))
}

export function countAvailableModels(models: AiModelCatalogItem[]): number {
  return models.filter((model) => model.is_available).length
}

export function modelCapabilityBadges(
  model: AiModelCatalogItem,
  t: Translate
): AiModelCapabilityBadge[] {
  const badges = new Map<string, AiModelCapabilityBadge>()
  const capabilitySource = metadataString(model.metadata, 'capability_source')
  const modelCapabilitiesAreInferred = capabilitySource === 'hermes_model_key_heuristic'

  for (const capability of metadataStringArray(model.metadata, 'runtime_capabilities')) {
    addCapabilityBadge(badges, capability, false, t)
  }
  for (const capability of providerMetadataCapabilities(model.metadata)) {
    addCapabilityBadge(badges, capability, false, t)
  }
  for (const capability of model.capabilities) {
    addCapabilityBadge(badges, capability, modelCapabilitiesAreInferred, t)
  }

  if (badges.size === 0) {
    return [
      {
        key: 'runtime-not-reported',
        label: t('Runtime did not report capabilities'),
        muted: true,
      },
    ]
  }

  return [...badges.values()]
}

export function modelRuntimeFacts(model: AiModelCatalogItem, t: Translate): AiModelRuntimeFact[] {
  const facts: AiModelRuntimeFact[] = []
  const details = metadataRecord(model.metadata, 'runtime_details') ?? metadataRecord(model.metadata, 'details')
  const infoSummary = metadataRecord(model.metadata, 'model_info_summary')
  const owner = metadataString(model.metadata, 'owned_by')
  const source = metadataString(model.metadata, 'source')
  const capabilitySource = metadataString(model.metadata, 'capability_source')
  const contextWindow = model.context_window ?? metadataNumber(infoSummary, 'context_window')
  const embeddingDimension = model.embedding_dimension ?? metadataNumber(infoSummary, 'embedding_dimension')

  if (contextWindow) {
    facts.push({ key: 'context', label: t('Context'), value: `${contextWindow} ctx` })
  }
  if (embeddingDimension) {
    facts.push({ key: 'embedding', label: t('Embedding'), value: `${embeddingDimension} dim` })
  }
  addRuntimeFact(facts, 'family', t('Family'), metadataString(details, 'family'))
  addRuntimeFact(facts, 'format', t('Format'), metadataString(details, 'format'))
  addRuntimeFact(facts, 'parameters', t('Parameters'), metadataString(details, 'parameter_size'))
  addRuntimeFact(facts, 'quantization', t('Quantization'), metadataString(details, 'quantization_level'))
  addRuntimeFact(facts, 'owner', t('Owner'), owner)
  addRuntimeFact(facts, 'source', t('Source'), source)
  addRuntimeFact(facts, 'capability-source', t('Capability source'), capabilitySource)

  return facts
}

export function modelDetail(model: AiModelCatalogItem): string {
  const details = [`${model.model_key}`, model.category, model.privacy]
  if (model.context_window) {
    details.push(`${model.context_window} ctx`)
  }
  if (model.embedding_dimension) {
    details.push(`${model.embedding_dimension} dim`)
  }
  return details.join(' · ')
}

function modelSearchTokens(model: AiModelCatalogItem, provider: AiProviderAccount): string[] {
  const details = metadataRecord(model.metadata, 'runtime_details') ?? metadataRecord(model.metadata, 'details')
  const tokens = [
    provider.display_name,
    provider.provider_key,
    provider.provider_kind,
    model.display_name,
    model.model_key,
    model.category,
    model.privacy,
    ...model.capabilities,
    ...metadataStringArray(model.metadata, 'runtime_capabilities'),
    ...providerMetadataCapabilities(model.metadata),
    metadataString(model.metadata, 'owned_by'),
    metadataString(model.metadata, 'source'),
    metadataString(model.metadata, 'capability_source'),
    metadataString(details, 'family'),
    metadataString(details, 'format'),
    metadataString(details, 'parameter_size'),
    metadataString(details, 'quantization_level'),
  ]
  const compacted: string[] = []
  for (const token of tokens) {
    if (token.trim().length > 0) compacted.push(token)
  }
  return compacted
}

function addCapabilityBadge(
  badges: Map<string, AiModelCapabilityBadge>,
  capability: string,
  muted: boolean,
  t: Translate
): void {
  const normalized = normalizedCapabilityKey(capability)
  if (!normalized) return
  const existing = badges.get(normalized)
  if (existing && (!existing.muted || muted)) return
  badges.set(normalized, {
    key: normalized,
    label: capabilityLabel(normalized, capability, t),
    muted,
  })
}

function normalizedCapabilityKey(capability: string): string {
  const normalized = capability.trim().replace(/_/g, '-').toLowerCase()
  if (!normalized) return ''
  if (['completion', 'completions', 'chat-completion', 'chat-completions', 'text-generation', 'text'].includes(normalized)) {
    return 'chat'
  }
  if (['embedding', 'embeddings'].includes(normalized)) return 'embeddings'
  if (['image', 'images', 'image-input'].includes(normalized)) return 'vision'
  if (['audio-input', 'audio-output'].includes(normalized)) return 'audio'
  if (['tool', 'tools', 'tool-use', 'function-calling'].includes(normalized)) return 'tools'
  return normalized
}

function capabilityLabel(capability: string, fallback: string, t: Translate): string {
  const labels: Record<string, string> = {
    audio: t('Audio'),
    chat: t('Chat'),
    embeddings: t('Embeddings'),
    extraction: t('Extraction'),
    multimodal: t('Multimodal'),
    reasoning: t('Reasoning'),
    routing: t('Routing'),
    summarization: t('Summaries'),
    tools: t('Tools'),
    vision: t('Vision'),
  }
  return labels[capability] ?? fallback
}

function addRuntimeFact(
  facts: AiModelRuntimeFact[],
  key: string,
  label: string,
  value: string
): void {
  if (!value) return
  facts.push({ key, label, value })
}

function providerMetadataCapabilities(metadata: Record<string, unknown>): string[] {
  const providerMetadata = metadataRecord(metadata, 'provider_metadata')
  if (!providerMetadata) return []
  return [
    ...metadataStringArray(providerMetadata, 'capabilities'),
    ...metadataStringArray(providerMetadata, 'modalities'),
    ...metadataStringArray(providerMetadata, 'input_modalities'),
    ...metadataStringArray(providerMetadata, 'output_modalities'),
    ...metadataStringArray(providerMetadata, 'supported_features'),
  ]
}

function metadataRecord(
  metadata: Record<string, unknown> | null,
  key: string
): Record<string, unknown> | null {
  if (!metadata) return null
  const value = metadata[key]
  return isRecord(value) ? value : null
}

function metadataString(metadata: Record<string, unknown> | null, key: string): string {
  if (!metadata) return ''
  const value = metadata[key]
  return typeof value === 'string' ? value : ''
}

function metadataNumber(metadata: Record<string, unknown> | null, key: string): number | null {
  if (!metadata) return null
  const value = metadata[key]
  return typeof value === 'number' ? value : null
}

function metadataStringArray(metadata: Record<string, unknown> | null, key: string): string[] {
  if (!metadata) return []
  const value = metadata[key]
  if (!Array.isArray(value)) return []
  const items: string[] = []
  for (const item of value) {
    if (typeof item === 'string' && item.trim().length > 0) items.push(item)
  }
  return items
}
