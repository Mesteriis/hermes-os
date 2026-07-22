import { errorMessage, parseRouteOptionValue } from './aiSettingsPresentation'

interface AiRouteActionDependencies {
  updateRoute: (variables: {
    slot: string
    request: { provider_id: string; model_key: string }
  }) => Promise<unknown>
  deleteRoute: (slot: string) => Promise<unknown>
  refreshOverview: () => Promise<unknown>
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: (key: string) => string
}

export async function updateAiRouteSelection(
  slot: string,
  value: string,
  dependencies: AiRouteActionDependencies
): Promise<void> {
  if (!value) {
    try {
      await dependencies.deleteRoute(slot)
      await dependencies.refreshOverview()
      dependencies.setActionMessage(dependencies.t('AI model route cleared'))
    } catch (error) {
      dependencies.setError(errorMessage(error, dependencies.t('AI model route delete failed')))
    }
    return
  }

  const parsed = parseRouteOptionValue(value)
  if (!parsed) return

  try {
    await dependencies.updateRoute({
      slot,
      request: {
        provider_id: parsed.providerId,
        model_key: parsed.modelKey
      }
    })
    await dependencies.refreshOverview()
    dependencies.setActionMessage(dependencies.t('AI model route updated'))
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI model route update failed')))
  }
}
