import { errorMessage } from './aiSettingsPresentation'
import type { AiModelCatalogItem } from '../types/aiControlCenter'

interface ModelAvailabilityDependencies {
  updateAvailability: (request: {
    provider_id: string
    model_key: string
    is_available: boolean
  }) => Promise<AiModelCatalogItem>
  refreshOverview: () => Promise<unknown>
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: (key: string) => string
}

export async function updateAiModelAvailability(
  model: AiModelCatalogItem,
  isAvailable: boolean,
  dependencies: ModelAvailabilityDependencies
): Promise<AiModelCatalogItem | null> {
  try {
    const updated = await dependencies.updateAvailability({
      provider_id: model.provider_id,
      model_key: model.model_key,
      is_available: isAvailable
    })
    await dependencies.refreshOverview()
    dependencies.setActionMessage(dependencies.t(isAvailable ? 'AI model enabled' : 'AI model disabled'))
    return updated
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI model availability update failed')))
    return null
  }
}
