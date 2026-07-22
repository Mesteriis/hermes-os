import { describe, expect, it, vi } from 'vitest'
import { toggleMailService } from './integrationMailActions'
import type { MailSyncSettings } from '../../../shared/mailSync/types'
import { DEFAULT_MAIL_SYNC_WINDOWS } from '../../../shared/mailSync/types'
import type { ProviderAccount } from '../types/settings'

describe('integration mail actions', () => {
  it('does not mutate while sync settings are unavailable', async () => {
    const dependencies = dependenciesFor()

    await toggleMailService(account(), true, null, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith('Mail sync settings are not loaded yet.')
    expect(dependencies.updateMailSyncSettings).not.toHaveBeenCalled()
  })

  it('updates sync enabled while preserving current runtime settings', async () => {
    const dependencies = dependenciesFor()
    const settings: MailSyncSettings = {
      account_id: 'account-1',
      sync_enabled: false,
      windows: 3,
      batch_size: 25,
      poll_interval_seconds: 120,
      failure_threshold: 4,
      updated_at: '2026-07-21T00:00:00Z'
    }

    await toggleMailService(account(), true, settings, dependencies)

    expect(dependencies.updateMailSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: true,
        batch_size: 25,
        windows: 3,
        poll_interval_seconds: 120,
        failure_threshold: 4
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail service enabled')
    expect(dependencies.setActiveAccount).toHaveBeenLastCalledWith(null)
  })

  it('normalizes malformed current settings before applying toggle', async () => {
    const dependencies = dependenciesFor()
    const settings: MailSyncSettings = {
      account_id: 'account-1',
      sync_enabled: false,
      windows: 0,
      batch_size: 12_345,
      poll_interval_seconds: 10,
      updated_at: '2026-07-21T00:00:00Z'
    }

    await toggleMailService(account(), true, settings, dependencies)

    expect(dependencies.updateMailSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: true,
        batch_size: 12_345,
        windows: DEFAULT_MAIL_SYNC_WINDOWS,
        poll_interval_seconds: 60,
        failure_threshold: 3,
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail service enabled')
  })
})

function dependenciesFor() {
  return {
    t: (key: string) => key,
    setActiveAccount: vi.fn(),
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateMailSyncSettings: vi.fn().mockResolvedValue({})
  }
}

function account(): ProviderAccount {
  return {
    account_id: 'account-1',
    provider_kind: 'gmail',
    display_name: 'Primary account',
    external_account_id: 'external-1',
    config: {},
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
