import type { AiProviderAccount } from '../types/aiControlCenter'

export function aiModelPickerDescription(
  provider: Pick<AiProviderAccount, 'display_name'> | null,
  availableModelCount: number,
  totalModelCount: number,
  translate: (key: string) => string
): string {
  if (!provider) return translate('Select a provider before choosing models.')
  return `${provider.display_name} · ${availableModelCount}/${totalModelCount} ${translate('available')}`
}
