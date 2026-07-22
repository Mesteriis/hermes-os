import type {
  AiModelCatalogItem,
  AiModelRoute,
  AiProviderAccount,
  AiProviderPreset
} from '../types/aiControlCenter'
import { aiProviderBrand, providerBrandClass } from './providerBranding'

export interface AiProviderListItem {
  id: string
  icon: string
  iconTone: string
  title: string
  subtitle: string
  badge: string
  metric: string
  provider: AiProviderAccount | null
  preset: AiProviderPreset | null
}

export type AiProviderListSelection =
  | { kind: 'provider'; providerId: string }
  | { kind: 'preset'; preset: AiProviderPreset }
  | null

export function aiProviderListSelection(item: {
  provider: Pick<AiProviderAccount, 'provider_id'> | null
  preset: AiProviderPreset | null
}): AiProviderListSelection {
  if (item.provider) return { kind: 'provider', providerId: item.provider.provider_id }
  if (item.preset) return { kind: 'preset', preset: item.preset }
  return null
}

export interface AiProviderListGroup {
  id: string
  label: string
  items: AiProviderListItem[]
}

type AiProviderListGroups = Record<'local' | 'cli' | 'api', AiProviderListItem[]>

export interface AiDetailRow {
  label: string
  value: string
}

type Translator = (key: string) => string

export type AiSettingsTab = 'providers' | 'models' | 'routes' | 'stats'

export interface AiSettingsTabModel {
  id: AiSettingsTab
  icon: string
  label: string
  count: number
}

export function buildAiSettingsTabs(
  counts: {
    providers: number
    models: number
    routes: number
    stats: number
  },
  t: Translator
): AiSettingsTabModel[] {
  return [
    { id: 'providers', icon: 'tabler:plug-connected', label: t('Provider setup'), count: counts.providers },
    { id: 'models', icon: 'tabler:list-search', label: t('Model catalog'), count: counts.models },
    { id: 'routes', icon: 'tabler:route', label: t('Action routing'), count: counts.routes },
    { id: 'stats', icon: 'tabler:chart-histogram', label: t('Usage statistics'), count: counts.stats },
  ]
}

export function buildAiProviderListGroups(
  providers: AiProviderAccount[],
  localPresets: AiProviderPreset[],
  models: AiModelCatalogItem[],
  providerForPreset: (preset: AiProviderPreset) => AiProviderAccount | null,
  t: Translator
): AiProviderListGroup[] {
  const groups: AiProviderListGroups = { local: [], cli: [], api: [] }
  for (const provider of providers) {
    groups[providerGroupId(provider.provider_kind)].push(
      providerListItemForProvider(provider, models, t)
    )
  }
  for (const preset of localPresets) {
    if (providerForPreset(preset)) continue
    groups[providerGroupId(preset.provider_kind)].push(providerListItemForPreset(preset, t))
  }
  return [
    { id: 'local', label: t('Local runtimes'), items: groups.local },
    { id: 'cli', label: t('CLI providers'), items: groups.cli },
    { id: 'api', label: t('Remote APIs'), items: groups.api }
  ]
}

export function buildAiSelectedProviderRows(
  provider: AiProviderAccount | null,
  baseUrl: (provider: AiProviderAccount) => string,
  t: Translator
): AiDetailRow[] {
  if (!provider) return []
  return [
    { label: t('Provider ID'), value: provider.provider_id },
    { label: t('Provider key'), value: provider.provider_key },
    { label: t('Provider kind'), value: provider.provider_kind },
    { label: t('Base URL'), value: baseUrl(provider) || t('No base URL') },
    { label: t('Consent'), value: provider.consent_state }
  ]
}

export function countProviderRoutes(providerId: string | null, routes: AiModelRoute[]): number {
  if (!providerId) return 0
  return routes.filter((route) => route.provider_id === providerId).length
}

export function providerIcon(providerKind: string, providerKey?: string): string {
  return aiProviderBrand(providerKind, providerKey).icon
}

export function providerIconTone(providerKind: string, providerKey?: string): string {
  return providerBrandClass(aiProviderBrand(providerKind, providerKey))
}

function providerGroupId(providerKind: string): 'local' | 'cli' | 'api' {
  if (providerKind === 'built_in') return 'local'
  if (providerKind === 'cli') return 'cli'
  return 'api'
}

function providerListItemForProvider(
  provider: AiProviderAccount,
  models: AiModelCatalogItem[],
  t: Translator
): AiProviderListItem {
  const modelCount = models.filter((model) => model.provider_id === provider.provider_id).length
  const brand = aiProviderBrand(provider.provider_kind, provider.provider_key)
  return {
    id: provider.provider_id,
    icon: brand.icon,
    iconTone: providerBrandClass(brand),
    title: provider.display_name,
    subtitle: `${provider.provider_key} · ${provider.status} · ${provider.consent_state}`,
    badge: provider.provider_kind,
    metric: modelCount > 0 ? `${modelCount} ${t('models')}` : t('No models synced'),
    provider,
    preset: null
  }
}

function providerListItemForPreset(preset: AiProviderPreset, t: Translator): AiProviderListItem {
  const brand = aiProviderBrand(preset.provider_kind, preset.provider_key)
  return {
    id: `${preset.provider_kind}:${preset.provider_key}`,
    icon: brand.icon,
    iconTone: providerBrandClass(brand),
    title: preset.display_name,
    subtitle: `${preset.privacy} · ${preset.capabilities.join(', ')}`,
    badge: t('Preset'),
    metric: t('Connect'),
    provider: null,
    preset
  }
}
