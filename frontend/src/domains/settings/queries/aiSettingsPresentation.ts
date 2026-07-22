import type {
  AiCapabilitySlot,
  AiModelCatalogItem,
  AiModelRoute,
  AiProviderAccount,
  AiProviderPreset
} from '../types/aiControlCenter'
import { SLOT_DESCRIPTIONS, SLOT_LABELS } from './aiSettingsCatalog'
import { modelUsableForSlot } from './aiSettingsPredicates'

export interface AiModelRouteRow {
  slot: AiCapabilitySlot
  label: string
  description: string
  selectedValue: string
  selectedModelLabel: string
  options: AiModelRouteOption[]
}

export interface AiModelRouteOption {
  value: string
  label: string
  detail: string
}

interface AiPresentationTranslator {
  (key: string): string
}

export const ROUTE_OPTION_SEPARATOR = '|'

export function routeOptionValue(providerId: string, modelKey: string): string {
  return `${encodeURIComponent(providerId)}${ROUTE_OPTION_SEPARATOR}${encodeURIComponent(modelKey)}`
}

export function parseRouteOptionValue(value: string): { providerId: string; modelKey: string } | null {
  const [providerId, modelKey] = value.split(ROUTE_OPTION_SEPARATOR)
  if (!providerId || !modelKey) return null
  return {
    providerId: decodeURIComponent(providerId),
    modelKey: decodeURIComponent(modelKey)
  }
}

export function mergedApiPresets(
  defaultApiProviderPresets: AiProviderPreset[],
  providerPresets: AiProviderPreset[]
): AiProviderPreset[] {
  const presetsByKey = new Map<string, AiProviderPreset>()
  for (const preset of defaultApiProviderPresets) {
    presetsByKey.set(preset.provider_key, preset)
  }
  for (const preset of providerPresets) {
    if (preset.provider_kind === 'api') {
      presetsByKey.set(preset.provider_key, preset)
    }
  }
  return Array.from(presetsByKey.values())
}

export function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message
  const message = typeof error === 'object' && error !== null && 'message' in error ? error.message : null
  return typeof message === 'string' ? message : fallback
}

export function modelStateKey(model: AiModelCatalogItem): string {
  return `${model.provider_id}:${model.model_key}`
}

export function resolveModelLabel(
  providers: Array<{ provider_id: string; display_name: string }>,
  models: Array<{ provider_id: string; model_key: string; display_name: string }>,
  providerId: string,
  modelKey: string
): string {
  const provider = providers.find((item) => item.provider_id === providerId)
  const model = models.find((item) => item.provider_id === providerId && item.model_key === modelKey)
  const providerName = provider?.display_name ?? providerId
  const modelName = model?.display_name ?? modelKey
  return `${providerName} / ${modelName}`
}

export function buildAiModelRouteRows(
  slots: AiCapabilitySlot[],
  routes: AiModelRoute[],
  providers: AiProviderAccount[],
  models: AiModelCatalogItem[],
  t: AiPresentationTranslator
): AiModelRouteRow[] {
  return slots.map((slot) => {
    const route = routes.find((candidate) => candidate.capability_slot === slot.slot) ?? null
    const options = models
      .filter((model) => modelUsableForSlot(model, slot))
      .map((model) => ({
        value: routeOptionValue(model.provider_id, model.model_key),
        label: resolveModelLabel(providers, models, model.provider_id, model.model_key),
        detail: `${model.privacy} · ${model.category}`
      }))
    return {
      slot,
      label: t(SLOT_LABELS[slot.slot] ?? slot.label),
      description: t(SLOT_DESCRIPTIONS[slot.slot] ?? slot.description),
      selectedValue: route ? routeOptionValue(route.provider_id, route.model_key) : '',
      selectedModelLabel: route
        ? resolveModelLabel(providers, models, route.provider_id, route.model_key)
        : t('Not routed'),
      options
    }
  })
}

export function modelRouteUsageCount(model: AiModelCatalogItem, routes: AiModelRoute[]): number {
  return routes.filter((route) => route.provider_id === model.provider_id && route.model_key === model.model_key).length
}

export function providerBaseUrl(provider: AiProviderAccount): string {
  const value = provider.config.base_url
  return typeof value === 'string' ? value : ''
}

export function modelDownloadProgressLabel(
  progress: number | null,
  t: AiPresentationTranslator
): string | null {
  if (progress === null) return null
  if (progress >= 100) return t('Finalizing model')
  if (progress >= 70) return t('Preparing model for routing')
  if (progress >= 30) return t('Downloading model')
  return t('Starting download')
}
