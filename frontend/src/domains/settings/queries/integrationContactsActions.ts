import type {
  AddressBookSyncRunResponse,
  ProviderAccountUpdate
} from '../api/settings'
import type { ProviderAccount } from '../types/settings'
import {
  accountConfigString,
  accountHasGoogleContactsWriteScope,
  accountSupportsContacts
} from './integrationAccountPredicates'

type ContactsTranslator = (key: string, params?: Record<string, string>) => string

interface ContactsActionDependencies {
  t: ContactsTranslator
  setActiveAccount: (accountId: string | null) => void
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateProviderAccount: (request: {
    accountId: string
    update: ProviderAccountUpdate
  }) => Promise<unknown>
  runAddressBookSyncNow: (accountId: string) => Promise<AddressBookSyncRunResponse>
}

export async function toggleContactsService(
  account: ProviderAccount,
  enabled: boolean,
  dependencies: ContactsActionDependencies
): Promise<void> {
  if (!ensureContactsAvailable(account, dependencies)) return

  await executeContactsAction(account, dependencies, {
    address_book_sync_enabled: enabled
  }, enabled ? dependencies.t('Contacts sync enabled') : dependencies.t('Contacts sync paused'))
}

export async function enableContactsBidirectional(
  account: ProviderAccount,
  dependencies: ContactsActionDependencies
): Promise<void> {
  if (!ensureContactsAvailable(account, dependencies)) return

  const canWrite = accountHasGoogleContactsWriteScope(account)
  await executeContactsAction(
    account,
    dependencies,
    {
      address_book_sync_enabled: true,
      address_book_sync_direction: 'bidirectional',
      address_book_remote_write_enabled: canWrite
    },
    canWrite
      ? dependencies.t('Contacts two-way sync enabled')
      : dependencies.t('Contacts two-way sync is prepared; reconnect with Contacts write scope to push changes.')
  )
}

export async function runContactsSyncNow(
  account: ProviderAccount,
  dependencies: ContactsActionDependencies
): Promise<void> {
  if (!ensureContactsAvailable(account, dependencies)) return

  dependencies.setActiveAccount(account.account_id)
  dependencies.clearMessages()
  try {
    const result = await dependencies.runAddressBookSyncNow(account.account_id)
    dependencies.setActionMessage(
      dependencies.t('Address book sync finished: {provider} provider entries, {local} local entries.', {
        provider: String(result.provider_entries_upserted),
        local: String(result.local_entries_pushed)
      })
    )
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : dependencies.t('Address book sync failed'))
  } finally {
    dependencies.setActiveAccount(null)
  }
}

function ensureContactsAvailable(
  account: ProviderAccount,
  dependencies: ContactsActionDependencies
): boolean {
  if (!accountSupportsContacts(account)) {
    dependencies.setError(dependencies.t('Contacts are not provided by this integration.'))
    return false
  }
  if (accountConfigString(account, 'address_book_sync_unsupported_reason')) {
    dependencies.setError(
      dependencies.t('Contacts sync is disabled for this account because the provider adapter is not available.')
    )
    return false
  }
  return true
}

async function executeContactsAction(
  account: ProviderAccount,
  dependencies: ContactsActionDependencies,
  update: ProviderAccountUpdate,
  successMessage: string
): Promise<void> {
  dependencies.setActiveAccount(account.account_id)
  dependencies.clearMessages()
  try {
    await dependencies.updateProviderAccount({ accountId: account.account_id, update })
    dependencies.setActionMessage(successMessage)
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : dependencies.t('Contacts sync update failed'))
  } finally {
    dependencies.setActiveAccount(null)
  }
}
