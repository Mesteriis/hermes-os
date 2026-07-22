import { describe, expect, it, vi } from 'vitest'
import { toggleSelectedAccount } from './integrationAccountToggleActions'
import type { ProviderAccount } from '../types/settings'

describe('integration account toggle actions', () => {
  it('opens the Gmail reconnect flow when enabling a selected account', async () => {
    const dependencies = dependenciesFor()

    await toggleSelectedAccount(account('gmail'), true, dependencies)

    expect(dependencies.openConnectWizard).toHaveBeenCalledWith('mail')
    expect(dependencies.logoutMailAccount).not.toHaveBeenCalled()
  })

  it('delegates disabling to the mail logout action', async () => {
    const dependencies = dependenciesFor()

    await toggleSelectedAccount(account('gmail'), false, dependencies)

    expect(dependencies.logoutMailAccount).toHaveBeenCalledWith('account-1')
  })

  it('rejects non-mail account toggle contracts', async () => {
    const dependencies = dependenciesFor()

    await toggleSelectedAccount(account('telegram_user'), true, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'This provider does not expose a generic account toggle contract yet.'
    )
    expect(dependencies.openConnectWizard).not.toHaveBeenCalled()
  })
})

function dependenciesFor() {
  return {
    t: (key: string) => key,
    openConnectWizard: vi.fn(),
    logoutMailAccount: vi.fn().mockResolvedValue(undefined),
    setError: vi.fn()
  }
}

function account(provider_kind: string): ProviderAccount {
  return {
    account_id: 'account-1',
    provider_kind,
    display_name: 'Primary account',
    external_account_id: 'external-1',
    config: {},
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
