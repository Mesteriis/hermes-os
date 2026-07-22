import { describe, expect, it, vi } from 'vitest'
import {
  enableContactsBidirectional,
  runContactsSyncNow,
  toggleContactsService
} from './integrationContactsActions'
import type { ProviderAccount } from '../types/settings'

describe('integration contacts actions', () => {
  it('rejects accounts without the contacts capability before mutation', async () => {
    const dependencies = dependenciesFor()

    await toggleContactsService(account({ connected_services: [] }), true, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith('Contacts are not provided by this integration.')
    expect(dependencies.updateProviderAccount).not.toHaveBeenCalled()
  })

  it('enables bidirectional sync with the account write scope', async () => {
    const dependencies = dependenciesFor()

    await enableContactsBidirectional(
      account({
        connected_services: ['contacts'],
        requested_scopes: ['https://www.googleapis.com/auth/contacts']
      }),
      dependencies
    )

    expect(dependencies.updateProviderAccount).toHaveBeenCalledWith({
      accountId: 'account-1',
      update: {
        address_book_sync_enabled: true,
        address_book_sync_direction: 'bidirectional',
        address_book_remote_write_enabled: true
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Contacts two-way sync enabled')
  })

  it('reports manual sync counters and clears active state', async () => {
    const dependencies = dependenciesFor()
    dependencies.runAddressBookSyncNow.mockResolvedValue({
      provider_entries_seen: 4,
      provider_entries_upserted: 3,
      provider_entries_skipped: 1,
      local_entries_seen: 2,
      local_entries_pushed: 1,
      local_entries_blocked: 1,
      status: 'completed'
    })

    await runContactsSyncNow(account({ connected_services: ['contacts'] }), dependencies)

    expect(dependencies.setActiveAccount).toHaveBeenNthCalledWith(1, 'account-1')
    expect(dependencies.setActiveAccount).toHaveBeenLastCalledWith(null)
    expect(dependencies.setActionMessage).toHaveBeenCalledWith(
      'Address book sync finished: 3 provider entries, 1 local entries.'
    )
  })
})

function dependenciesFor() {
  return {
    t: (key: string, params?: Record<string, string>) =>
      params ? key.replace('{provider}', params.provider ?? '').replace('{local}', params.local ?? '') : key,
    setActiveAccount: vi.fn(),
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateProviderAccount: vi.fn().mockResolvedValue({}),
    runAddressBookSyncNow: vi.fn()
  }
}

function account(config: Record<string, unknown>): ProviderAccount {
  return {
    account_id: 'account-1',
    provider_kind: 'gmail',
    display_name: 'Primary account',
    external_account_id: 'external-1',
    config,
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
