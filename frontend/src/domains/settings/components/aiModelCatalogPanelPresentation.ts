import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'
import { modelMatchesSearch } from './aiModelCatalogPresentation'
import { aiProviderBrand, providerBrandClass } from './providerBranding'

export interface AiProviderModelGroup {
  provider: AiProviderAccount
  models: AiModelCatalogItem[]
  availableCount: number
}

export function buildProviderModelGroups(
  providers: AiProviderAccount[],
  models: AiModelCatalogItem[]
): AiProviderModelGroup[] {
  return providers.map((provider) => {
    const providerModels = models.filter((model) => model.provider_id === provider.provider_id)
    return {
      provider,
      models: providerModels,
      availableCount: providerModels.filter((model) => model.is_available).length
    }
  })
}

export function findSelectedProviderModelGroup(
  groups: AiProviderModelGroup[],
  activeProviderId: string | null
): AiProviderModelGroup | null {
  if (!groups.length) return null
  return groups.find((group) => group.provider.provider_id === activeProviderId) ?? groups[0]
}

export function filterProviderModels(
  group: AiProviderModelGroup | null,
  search: string,
  availableOnly: boolean
): AiModelCatalogItem[] {
  if (!group) return []
  const query = search.trim().toLowerCase()
  return group.models.filter((model) =>
    (!availableOnly || model.is_available) &&
    (!query || modelMatchesSearch(model, group.provider, query))
  )
}

export function providerIcon(providerKind: string, providerKey?: string): string {
  return aiProviderBrand(providerKind, providerKey).icon
}

export function providerIconTone(providerKind: string, providerKey?: string): string {
  return providerBrandClass(aiProviderBrand(providerKind, providerKey))
}
