import { describe, expect, it, vi } from 'vitest'
import { saveMailSyncSettings, toggleMailSync } from './communicationsMailSyncActions'
import {
  DEFAULT_MAIL_SYNC_WINDOWS,
  MAX_MAIL_BATCH_SIZE,
  MAX_MAIL_SYNC_WINDOWS,
  type MailSyncSettings
} from '../../../shared/mailSync/types'

describe('communications mail sync actions', () => {
  it('rejects non-positive draft values without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '0', '300', '10', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'Mail sync settings must be within allowed ranges.'
    )
    expect(dependencies.updateSyncSettings).not.toHaveBeenCalled()
  })

  it('rejects oversized batch size values without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), `${MAX_MAIL_BATCH_SIZE + 1}`, '300', '10', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'Mail sync settings must be within allowed ranges.'
    )
    expect(dependencies.updateSyncSettings).not.toHaveBeenCalled()
  })

  it('rejects out-of-range poll interval values without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '50', '59', '10', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'Mail sync settings must be within allowed ranges.'
    )
    expect(dependencies.updateSyncSettings).not.toHaveBeenCalled()
  })

  it('rejects out-of-range windows values without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '50', '300', `${MAX_MAIL_SYNC_WINDOWS + 1}`, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'Mail sync settings must be within allowed ranges.'
    )
    expect(dependencies.updateSyncSettings).not.toHaveBeenCalled()
  })

  it('rejects non-positive windows values without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '50', '300', '0', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'Mail sync settings must be within allowed ranges.'
    )
    expect(dependencies.updateSyncSettings).not.toHaveBeenCalled()
  })

  it('accepts windows at the maximum limit', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '50', '300', `${MAX_MAIL_SYNC_WINDOWS}`, dependencies)

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: false,
        batch_size: 50,
        windows: MAX_MAIL_SYNC_WINDOWS,
        poll_interval_seconds: 300,
        failure_threshold: 4
      }
    })
  })

  it('saves validated draft values and preserves current sync state', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings('account-1', settings(), '50', '120', '3', dependencies)

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: false,
        batch_size: 50,
        windows: 3,
        poll_interval_seconds: 120,
        failure_threshold: 4
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail sync settings saved')
  })

  it('accepts the maximum allowed batch size', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings(
      'account-1',
      settings(),
      `${MAX_MAIL_BATCH_SIZE}`,
      '300',
      `${MAX_MAIL_SYNC_WINDOWS}`,
      dependencies
    )

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: false,
        batch_size: MAX_MAIL_BATCH_SIZE,
        windows: MAX_MAIL_SYNC_WINDOWS,
        poll_interval_seconds: 300,
        failure_threshold: 4
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail sync settings saved')
  })

  it('accepts the maximum allowed poll interval', async () => {
    const dependencies = dependenciesFor()

    await saveMailSyncSettings(
      'account-1',
      settings(),
      '50',
      '86400',
      `${MAX_MAIL_SYNC_WINDOWS}`,
      dependencies
    )

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: false,
        batch_size: 50,
        windows: MAX_MAIL_SYNC_WINDOWS,
        poll_interval_seconds: 86400,
        failure_threshold: 4
      }
    })
  })

  it('toggles sync while retaining the existing batch configuration', async () => {
    const dependencies = dependenciesFor()

    await toggleMailSync('account-1', settings(), true, dependencies)

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: true,
        windows: 3,
        batch_size: 25,
        poll_interval_seconds: 300,
        failure_threshold: 4
      }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail sync enabled')
  })

  it('normalizes current settings windows when toggling sync', async () => {
    const dependencies = dependenciesFor()
    const withInvalidWindows = settings()
    withInvalidWindows.windows = 0

    await toggleMailSync('account-1', withInvalidWindows, true, dependencies)

    expect(dependencies.updateSyncSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: {
        sync_enabled: true,
        windows: DEFAULT_MAIL_SYNC_WINDOWS,
        batch_size: 25,
        poll_interval_seconds: 300,
        failure_threshold: 4
      }
    })
  })
})

function dependenciesFor() {
  return {
    t: (key: string) => key,
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateSyncSettings: vi.fn().mockResolvedValue({})
  }
}

function settings(): MailSyncSettings {
  return {
    account_id: 'account-1',
    sync_enabled: false,
    windows: 3,
    batch_size: 25,
    poll_interval_seconds: 300,
    failure_threshold: 4,
    updated_at: '2026-07-21T00:00:00Z'
  }
}
