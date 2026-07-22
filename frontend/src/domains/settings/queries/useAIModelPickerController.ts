import { computed } from 'vue'
import type { AISettingsSurface } from './useAISettingsSurface'
import { useI18n } from '../../../platform/i18n'
import type { AiModelCatalogItem } from '../types/aiControlCenter'
import { countAvailableModels } from '../components/aiModelCatalogPresentation'
import { aiModelPickerDescription } from '../components/aiModelPickerPresentation'

export function useAIModelPickerController(options: {
  surface: AISettingsSurface
}) {
  const { t } = useI18n()

  const selectedAvailableModelCount = computed(() => countAvailableModels(
    options.surface.selectedProviderModels.value
  ))

  const modelPickerDescription = computed(() => aiModelPickerDescription(
    options.surface.selectedProvider.value,
    selectedAvailableModelCount.value,
    options.surface.selectedProviderModels.value.length,
    t,
  ))

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleToggleModelAvailability(model: AiModelCatalogItem, event: Event): void {
    void options.surface.handleModelAvailability(model, eventChecked(event))
  }

  function handleSyncModels(): void {
    const provider = options.surface.selectedProvider.value
    if (!provider) return
    void options.surface.handleSyncModels(provider)
  }

  function handleDownloadModel(model: AiModelCatalogItem): void {
    void options.surface.handleModelDownload(model)
  }

  function modelProgress(model: AiModelCatalogItem): number {
    return options.surface.modelDownloadProgressValue(model) ?? 0
  }

  function modelProgressLabel(model: AiModelCatalogItem): string {
    return options.surface.modelDownloadProgressLabel(model) ?? t('Downloading model')
  }

  return {
    t,
    selectedAvailableModelCount,
    modelPickerDescription,
    handleSyncModels,
    handleDownloadModel,
    handleToggleModelAvailability,
    modelProgress,
    modelProgressLabel,
  }
}
