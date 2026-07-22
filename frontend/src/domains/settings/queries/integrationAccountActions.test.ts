import { describe, expect, it, vi } from 'vitest'
import {
  deleteMailAccount,
  exportMailAccount,
  logoutMailAccount,
  saveAccountLabel
} from './integrationAccountActions'
import type { ProviderAccount } from '../types/settings'

describe('integration account actions', () => {
  it('rejects an empty account label without mutation', async () => {
    const dependencies = dependenciesFor()

    await saveAccountLabel(account(), '   ', dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith('Label cannot be empty')
    expect(dependencies.updateProviderAccount).not.toHaveBeenCalled()
  })

  it('exports the account snapshot through the download boundary', async () => {
    const dependencies = dependenciesFor()
    dependencies.exportMailAccount.mockResolvedValue({ exported_at: '2026-07-21T12:00:00Z' })

    await exportMailAccount('account-1', dependencies)

    expect(dependencies.downloadJsonFile).toHaveBeenCalledWith(
      'mail-account-account-1-2026-07-21T12:00:00Z.json',
      { exported_at: '2026-07-21T12:00:00Z' }
    )
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail account export snapshot prepared')
  })

  it('logs out and deletes through the lifecycle mutation boundaries', async () => {
    const dependencies = dependenciesFor()
    dependencies.deleteMailAccount.mockResolvedValue({ vault_deleted_secret_refs: ['vault-ref-1'] })

    await logoutMailAccount('account-1', dependencies)
    await deleteMailAccount('account-1', dependencies)

    expect(dependencies.logoutMailAccount).toHaveBeenCalledWith('account-1')
    expect(dependencies.clearSelectedAccount).toHaveBeenCalledWith('account-1')
    expect(dependencies.setActionMessage).toHaveBeenLastCalledWith('Mail account deleted from Hermes and vault')
    expect(dependencies.setActiveAccount).toHaveBeenLastCalledWith(null)
  })
})

function dependenciesFor() {
  return {
    t: (key: string) => key,
    setActiveAccount: vi.fn(),
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    clearSelectedAccount: vi.fn(),
    updateProviderAccount: vi.fn().mockResolvedValue({}),
    exportMailAccount: vi.fn(),
    logoutMailAccount: vi.fn().mockResolvedValue({}),
    deleteMailAccount: vi.fn(),
    downloadJsonFile: vi.fn()
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
