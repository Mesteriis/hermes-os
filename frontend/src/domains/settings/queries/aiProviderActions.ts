import { errorMessage } from './aiSettingsPresentation'
import type {
  AiProviderAccount,
  AiProviderCommandResponse
} from '../types/aiControlCenter'

interface ProviderActionTranslator {
  (key: string): string
}

interface ProviderUpdateDependencies {
  updateProvider: (variables: {
    providerId: string
    request: { enabled: boolean }
  }) => Promise<AiProviderAccount>
  setSelectedProvider: (providerId: string) => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: ProviderActionTranslator
}

export async function toggleAiProvider(
  provider: AiProviderAccount,
  enabled: boolean,
  dependencies: ProviderUpdateDependencies
): Promise<AiProviderAccount | null> {
  try {
    const updated = await dependencies.updateProvider({
      providerId: provider.provider_id,
      request: { enabled }
    })
    dependencies.setSelectedProvider(updated.provider_id)
    dependencies.setActionMessage(dependencies.t(enabled ? 'AI provider enabled' : 'AI provider disabled'))
    return updated
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI provider update failed')))
    return null
  }
}

interface ConsentDependencies {
  updateConsent: (variables: {
    providerId: string
    request: { consented: boolean }
  }) => Promise<AiProviderAccount>
  setSelectedProvider: (providerId: string) => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: ProviderActionTranslator
}

export async function updateAiProviderConsent(
  provider: AiProviderAccount,
  consented: boolean,
  dependencies: ConsentDependencies
): Promise<AiProviderAccount | null> {
  try {
    const updated = await dependencies.updateConsent({
      providerId: provider.provider_id,
      request: { consented }
    })
    dependencies.setSelectedProvider(updated.provider_id)
    dependencies.setActionMessage(dependencies.t(
      consented ? 'Remote context consent granted' : 'Remote context consent revoked'
    ))
    return updated
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI provider consent update failed')))
    return null
  }
}

interface ProviderCommandDependencies {
  execute: (providerId: string) => Promise<AiProviderCommandResponse>
  refreshOverview?: () => Promise<unknown>
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  t: ProviderActionTranslator
}

export async function testAiProvider(
  provider: AiProviderAccount,
  dependencies: ProviderCommandDependencies
): Promise<AiProviderCommandResponse | null> {
  try {
    const response = await dependencies.execute(provider.provider_id)
    dependencies.setActionMessage(response.message || dependencies.t('AI provider test completed'))
    return response
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI provider test failed')))
    return null
  }
}

export async function syncAiProviderModels(
  provider: AiProviderAccount,
  dependencies: ProviderCommandDependencies & { refreshOverview: () => Promise<unknown> }
): Promise<AiProviderCommandResponse | null> {
  try {
    const response = await dependencies.execute(provider.provider_id)
    await dependencies.refreshOverview()
    dependencies.setActionMessage(response.message || dependencies.t('AI models synchronized'))
    return response
  } catch (error) {
    dependencies.setError(errorMessage(error, dependencies.t('AI model sync failed')))
    return null
  }
}
