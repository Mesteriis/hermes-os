import type { CalendarAccount } from '../types/settings'

type CalendarTranslator = (key: string) => string

interface CalendarActionDependencies {
  t: CalendarTranslator
  setActiveAccount: (accountId: string | null) => void
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateCalendarAccount: (request: {
    accountId: string
    update: { sync_status: 'active' | 'paused' }
  }) => Promise<unknown>
}

export async function toggleCalendarService(
  calendarAccount: CalendarAccount | null,
  enabled: boolean,
  dependencies: CalendarActionDependencies
): Promise<void> {
  if (!calendarAccount) {
    dependencies.setError(
      dependencies.t('No matching calendar account contract is available for this provider account.')
    )
    return
  }

  dependencies.setActiveAccount(calendarAccount.account_id)
  dependencies.clearMessages()
  try {
    await dependencies.updateCalendarAccount({
      accountId: calendarAccount.account_id,
      update: { sync_status: enabled ? 'active' : 'paused' }
    })
    dependencies.setActionMessage(
      enabled ? dependencies.t('Calendar service enabled') : dependencies.t('Calendar service paused')
    )
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : dependencies.t('Calendar service update failed')
    )
  } finally {
    dependencies.setActiveAccount(null)
  }
}
