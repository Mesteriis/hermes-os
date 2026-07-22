import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'

type Translate = (key: string) => string

export function findConnectedProvider(
  providers: AiProviderAccount[],
  providerId: string | null
): AiProviderAccount | null {
  if (!providerId) return null
  return providers.find((provider) => provider.provider_id === providerId) ?? null
}

export function connectedProviderModels(
  models: AiModelCatalogItem[],
  providerId: string | null
): AiModelCatalogItem[] {
  if (!providerId) return []
  return models.filter((model) => model.provider_id === providerId)
}

export function wizardNextLabel(step: number, t: Translate): string {
  if (step === 1) return t('Подключить')
  if (step === 2) return t('Проверить')
  return t('Готово')
}

export function aiProviderWizardVerificationSucceeded(status: string): boolean {
  return status === 'ok'
}
