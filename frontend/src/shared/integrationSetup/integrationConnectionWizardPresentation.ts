import type { ConnectionProviderId } from '../stores/integrationConnectionWizard'

export function canAdvanceIntegrationConnectionWizard(
  step: number,
  canSubmit: boolean,
  providerId: ConnectionProviderId,
  checksVisible: boolean
): boolean {
  if (step === 1) return canSubmit
  if (step === 2 && providerId === 'telegram') return checksVisible
  return true
}

export function integrationProviderIconTone(providerId: ConnectionProviderId): string {
  return `app-connection-wizard__provider-icon--${providerId}`
}

export function integrationCheckIcon(status: 'ready' | 'pending' | 'blocked'): string {
  if (status === 'ready') return 'tabler:check'
  if (status === 'blocked') return 'tabler:alert-triangle'
  return 'tabler:clock'
}
