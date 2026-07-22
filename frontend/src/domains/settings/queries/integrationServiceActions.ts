import type { CalendarAccount, ProviderAccount } from '../types/settings'
import type { AccountServiceRow } from './integrationAccountPresentation'

interface ServiceDispatchDependencies {
  selectedCalendarAccount: CalendarAccount | null
  toggleMail: (account: ProviderAccount, enabled: boolean) => Promise<void>
  toggleCalendar: (account: CalendarAccount | null, enabled: boolean) => Promise<void>
  toggleContacts: (account: ProviderAccount, enabled: boolean) => Promise<void>
  runContactsSync: (account: ProviderAccount) => Promise<void>
  setError: (message: string) => void
  unsupportedMessage: string
}

export async function toggleSelectedIntegrationService(
  account: ProviderAccount | null,
  serviceId: AccountServiceRow['id'],
  enabled: boolean,
  dependencies: ServiceDispatchDependencies
): Promise<void> {
  if (!account) return
  if (serviceId === 'mail') return dependencies.toggleMail(account, enabled)
  if (serviceId === 'calendar') return dependencies.toggleCalendar(dependencies.selectedCalendarAccount, enabled)
  if (serviceId === 'contacts') return dependencies.toggleContacts(account, enabled)
  dependencies.setError(dependencies.unsupportedMessage)
}

export async function runSelectedIntegrationService(
  account: ProviderAccount | null,
  serviceId: AccountServiceRow['id'],
  dependencies: ServiceDispatchDependencies
): Promise<void> {
  if (!account) return
  if (serviceId === 'contacts') return dependencies.runContactsSync(account)
  dependencies.setError(dependencies.unsupportedMessage)
}

export function runSelectedIntegrationServiceModeAction(
  serviceId: AccountServiceRow['id'],
  runContactsBidirectional: () => Promise<void>
): Promise<void> | undefined {
  if (serviceId === 'contacts') return runContactsBidirectional()
  return undefined
}
