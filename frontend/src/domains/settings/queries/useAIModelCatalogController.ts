import { computed, ref } from 'vue'
import type { AISettingsSurface } from './useAISettingsSurface'
import { useI18n } from '../../../platform/i18n'
import type { AiModelCatalogItem } from '../types/aiControlCenter'
import {
  buildProviderModelGroups,
  filterProviderModels,
  findSelectedProviderModelGroup,
  type AiProviderModelGroup,
} from '../components/aiModelCatalogPanelPresentation'

export function useAIModelCatalogController(options: {
  surface: AISettingsSurface
}) {
  const { t } = useI18n()

  const activeModelProviderId = ref<string | null>(null)
  const modelCatalogSearch = ref('')
  const showAvailableModelsOnly = ref(false)

  const providerModelGroups = computed<AiProviderModelGroup[]>(() => buildProviderModelGroups(
    options.surface.providers.value,
    options.surface.models.value
  ))

  const selectedModelGroup = computed<AiProviderModelGroup | null>(() => findSelectedProviderModelGroup(
    providerModelGroups.value,
    activeModelProviderId.value
  ))

  const selectedModelGroupModels = computed<AiModelCatalogItem[]>(() => filterProviderModels(
    selectedModelGroup.value,
    modelCatalogSearch.value,
    showAvailableModelsOnly.value
  ))

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleAvailableModelsFilterChange(event: Event): void {
    showAvailableModelsOnly.value = eventChecked(event)
  }

  function handleSelectModelProvider(providerId: string): void {
    activeModelProviderId.value = providerId
    modelCatalogSearch.value = ''
  }

  function handleSyncModels(): void {
    const group = selectedModelGroup.value
    if (!group) return
    void options.surface.handleSyncModels(group.provider)
  }

  function handleDownloadModel(model: AiModelCatalogItem): void {
    void options.surface.handleModelDownload(model)
  }

  function handleToggleModelAvailability(model: AiModelCatalogItem, event: Event): void {
    void options.surface.handleModelAvailability(model, eventChecked(event))
  }

  function modelProgress(model: AiModelCatalogItem): number {
    return options.surface.modelDownloadProgressValue(model) ?? 0
  }

  function modelProgressLabel(model: AiModelCatalogItem): string {
    return options.surface.modelDownloadProgressLabel(model) ?? t('Downloading model')
  }

  return {
    t,
    activeModelProviderId,
    modelCatalogSearch,
    showAvailableModelsOnly,
    providerModelGroups,
    selectedModelGroup,
    selectedModelGroupModels,
    handleAvailableModelsFilterChange,
    handleSyncModels,
    handleSelectModelProvider,
    handleDownloadModel,
    handleToggleModelAvailability,
    modelProgress,
    modelProgressLabel,
  }
}
