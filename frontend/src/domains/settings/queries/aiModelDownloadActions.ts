import { errorMessage } from './aiSettingsPresentation'
import type { AiModelCatalogItem } from '../types/aiControlCenter'

interface ModelDownloadDependencies {
  isDownloading: (key: string) => boolean
  startProgress: (key: string) => void
  finishProgress: (key: string) => void
  clearProgress: (key: string) => void
  download: (request: { provider_id: string; model_key: string }) => Promise<AiModelCatalogItem>
  refreshOverview: () => Promise<unknown>
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: (key: string) => string
}

export async function downloadAiModel(
  model: AiModelCatalogItem,
  key: string,
  dependencies: ModelDownloadDependencies
): Promise<AiModelCatalogItem | null> {
  if (dependencies.isDownloading(key)) return null

  dependencies.startProgress(key)
  try {
    const updated = await dependencies.download({
      provider_id: model.provider_id,
      model_key: model.model_key
    })
    dependencies.finishProgress(key)
    await dependencies.refreshOverview()
    dependencies.setActionMessage(dependencies.t('AI model downloaded'))
    return updated
  } catch (error) {
    dependencies.clearProgress(key)
    dependencies.setError(errorMessage(error, dependencies.t('AI model download failed')))
    return null
  }
}
