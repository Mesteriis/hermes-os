import { describe, expect, it, vi } from 'vitest'
import {
  runSelectedIntegrationService,
  runSelectedIntegrationServiceModeAction,
  toggleSelectedIntegrationService
} from './integrationServiceActions'
import type { ProviderAccount } from '../types/settings'

describe('integration service actions', () => {
  it('dispatches mail toggle to the mail action', async () => {
    const dependencies = dependenciesFor()

    await toggleSelectedIntegrationService(account(), 'mail', true, dependencies)

    expect(dependencies.toggleMail).toHaveBeenCalledWith(account(), true)
    expect(dependencies.toggleContacts).not.toHaveBeenCalled()
  })

  it('dispatches contacts manual sync and rejects unsupported services', async () => {
    const dependencies = dependenciesFor()

    await runSelectedIntegrationService(account(), 'contacts', dependencies)
    await runSelectedIntegrationService(account(), 'calendar', dependencies)

    expect(dependencies.runContactsSync).toHaveBeenCalledWith(account())
    expect(dependencies.setError).toHaveBeenCalledWith('unsupported')
  })

  it('dispatches the contacts mode action without view branching', async () => {
    const run = vi.fn().mockResolvedValue(undefined)

    await runSelectedIntegrationServiceModeAction('contacts', run)
    await runSelectedIntegrationServiceModeAction('mail', run)

    expect(run).toHaveBeenCalledOnce()
  })
})

function dependenciesFor() {
  return {
    selectedCalendarAccount: null,
    toggleMail: vi.fn().mockResolvedValue(undefined),
    toggleCalendar: vi.fn().mockResolvedValue(undefined),
    toggleContacts: vi.fn().mockResolvedValue(undefined),
    runContactsSync: vi.fn().mockResolvedValue(undefined),
    setError: vi.fn(),
    unsupportedMessage: 'unsupported'
  }
}

function account(): ProviderAccount {
  return {
    account_id: 'account-1', provider_kind: 'gmail', display_name: 'Account',
    external_account_id: 'external-1', config: {}, created_at: '', updated_at: ''
  }
}
