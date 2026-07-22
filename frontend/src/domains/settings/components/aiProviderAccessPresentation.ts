import type { AiProviderAccount } from '../types/aiControlCenter'

export function isAiProviderEnabled(
  provider: Pick<AiProviderAccount, 'status'>
): boolean {
  return provider.status !== 'disabled'
}

export function supportsAiRemoteContext(
  provider: Pick<AiProviderAccount, 'provider_kind'>
): boolean {
  return provider.provider_kind === 'api'
}

export function hasAiRemoteContextConsent(
  provider: Pick<AiProviderAccount, 'consent_state'>
): boolean {
  return provider.consent_state === 'granted'
}
