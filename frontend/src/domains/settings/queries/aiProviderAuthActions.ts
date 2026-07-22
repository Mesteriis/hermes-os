import { errorMessage } from './aiSettingsPresentation'
import type {
  AiProviderAuthStatusResponse,
  AiProviderAuthStartResponse,
  AiProviderPreset
} from '../types/aiControlCenter'

interface AuthStartDependencies {
  startAuth: (preset: AiProviderPreset) => Promise<AiProviderAuthStartResponse>
  setActiveAuth: (response: AiProviderAuthStartResponse) => void
  setSelectedProvider: (providerId: string) => void
  stopPolling: () => void
  startPolling: (setupId: string) => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: (key: string) => string
}

export async function startAiProviderAuth(
  preset: AiProviderPreset,
  dependencies: AuthStartDependencies
): Promise<AiProviderAuthStartResponse | null> {
  try {
    const response = await dependencies.startAuth(preset)
    dependencies.setActiveAuth(response)
    if (response.provider) {
      dependencies.setSelectedProvider(response.provider.provider_id)
      dependencies.stopPolling()
      dependencies.setActionMessage(dependencies.t('AI provider connected'))
      return response
    }
    dependencies.startPolling(response.setup_id)
    dependencies.setActionMessage(response.message || dependencies.t('AI provider callback started'))
    return response
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI provider callback start failed')))
    return null
  }
}

interface AuthStatusDependencies {
  isPending: () => boolean
  fetchStatus: (setupId: string) => Promise<AiProviderAuthStatusResponse>
  setActiveAuth: (response: AiProviderAuthStatusResponse) => void
  setSelectedProvider: (providerId: string) => void
  stopPolling: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: (key: string) => string
}

export async function refreshAiProviderAuthStatus(
  setupId: string,
  notify: boolean,
  dependencies: AuthStatusDependencies
): Promise<void> {
  if (dependencies.isPending()) return
  try {
    const response = await dependencies.fetchStatus(setupId)
    dependencies.setActiveAuth(response)
    if (response.provider) {
      dependencies.setSelectedProvider(response.provider.provider_id)
      dependencies.stopPolling()
      dependencies.setActionMessage(dependencies.t('AI provider connected'))
      return
    }
    if (notify) {
      dependencies.setActionMessage(response.message || dependencies.t('AI provider callback still waiting'))
    }
  } catch (error) {
    dependencies.stopPolling()
    dependencies.setError(errorMessage(error, dependencies.t('AI provider callback status failed')))
  }
}
