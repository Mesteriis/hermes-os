import type { ProviderAccountUpdate } from '../api/settings'
import type { ProviderAccount } from '../types/settings'

type AccountTranslator = (key: string, params?: Record<string, string>) => string

interface AccountActionDependencies {
  t: AccountTranslator
  setActiveAccount: (accountId: string | null) => void
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  clearSelectedAccount: (accountId: string) => void
  updateProviderAccount: (request: {
    accountId: string
    update: ProviderAccountUpdate
  }) => Promise<unknown>
  exportMailAccount: (accountId: string) => Promise<{ exported_at: string } & Record<string, unknown>>
  logoutMailAccount: (accountId: string) => Promise<unknown>
  deleteMailAccount: (accountId: string) => Promise<{ vault_deleted_secret_refs: string[] }>
  downloadJsonFile: (filename: string, value: unknown) => void
}

export async function saveAccountLabel(
  account: ProviderAccount | null,
  labelDraft: string,
  dependencies: AccountActionDependencies
): Promise<void> {
  if (!account) return
  const displayName = labelDraft.trim()
  if (!displayName) {
    dependencies.setError(dependencies.t('Label cannot be empty'))
    return
  }
  if (displayName === account.display_name.trim()) return

  dependencies.clearMessages()
  try {
    await dependencies.updateProviderAccount({
      accountId: account.account_id,
      update: { display_name: displayName }
    })
    dependencies.setActionMessage(dependencies.t('Account label saved'))
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : dependencies.t('Account label update failed')
    )
  }
}

export async function exportMailAccount(
  accountId: string,
  dependencies: AccountActionDependencies
): Promise<void> {
  await executeAccountAction(accountId, dependencies, async () => {
    const result = await dependencies.exportMailAccount(accountId)
    dependencies.downloadJsonFile(`mail-account-${accountId}-${result.exported_at}.json`, result)
    dependencies.setActionMessage(dependencies.t('Mail account export snapshot prepared'))
  }, 'Export failed')
}

export async function logoutMailAccount(
  accountId: string,
  dependencies: AccountActionDependencies
): Promise<void> {
  await executeAccountAction(accountId, dependencies, async () => {
    await dependencies.logoutMailAccount(accountId)
    dependencies.setActionMessage(dependencies.t('Mail account logged out'))
  }, 'Logout failed')
}

export async function deleteMailAccount(
  accountId: string,
  dependencies: AccountActionDependencies
): Promise<void> {
  await executeAccountAction(accountId, dependencies, async () => {
    const result = await dependencies.deleteMailAccount(accountId)
    dependencies.clearSelectedAccount(accountId)
    dependencies.setActionMessage(
      result.vault_deleted_secret_refs.length > 0
        ? dependencies.t('Mail account deleted from Hermes and vault')
        : dependencies.t('Mail account deleted')
    )
  }, 'Delete failed')
}

async function executeAccountAction(
  accountId: string,
  dependencies: AccountActionDependencies,
  action: () => Promise<void>,
  fallbackError: string
): Promise<void> {
  dependencies.setActiveAccount(accountId)
  try {
    await action()
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : dependencies.t(fallbackError))
  } finally {
    dependencies.setActiveAccount(null)
  }
}
