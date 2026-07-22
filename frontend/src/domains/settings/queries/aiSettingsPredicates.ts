import type { AiCapabilitySlot, AiModelCatalogItem } from '../types/aiControlCenter'

export function modelUsableForSlot(model: AiModelCatalogItem, slot: AiCapabilitySlot): boolean {
  if (!model.is_available) return false

  if (slot.requires_embedding_dimension && model.embedding_dimension !== slot.requires_embedding_dimension) {
    return false
  }

  if (slot.slot === 'embeddings') {
    return model.category === 'embeddings' || model.capabilities.includes('embeddings')
  }

  if (model.category === 'embeddings' || model.capabilities.includes('embeddings')) {
    return false
  }

  if (slot.slot === 'reasoning') {
    return model.category === 'reasoning' || model.capabilities.includes('reasoning')
  }

  if (slot.slot === 'summarization') {
    return model.capabilities.includes('summarization') || model.capabilities.includes('chat')
  }

  if (slot.slot === 'extraction') {
    return model.capabilities.includes('extraction') || model.capabilities.includes('chat')
  }

  return model.capabilities.includes('chat') || model.category === 'chat' || model.category === 'reasoning'
}

export function metadataBoolean(metadata: Record<string, unknown>, key: string): boolean {
  return metadata[key] === true
}

export function providerIsBuiltInOllama(providerId: string): boolean {
  return providerId === 'provider:built_in:ollama'
}
