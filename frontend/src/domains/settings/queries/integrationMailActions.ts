import type { MailSyncSettings, MailSyncSettingsUpdate } from '../../../shared/mailSync/types'
import type { ProviderAccount } from '../types/settings'
import { isMailProvider } from './integrationAccountPredicates'
import { normalizeMailSyncSettingsValues } from './mailSyncNormalization'

type MailTranslator = (key: string) => string

interface MailActionDependencies {
  t: MailTranslator
  setActiveAccount: (accountId: string | null) => void
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateMailSyncSettings: (request: {
    accountId: string
    settings: MailSyncSettingsUpdate
  }) => Promise<unknown>
}

export async function toggleMailService(
  account: ProviderAccount,
  enabled: boolean,
  current: MailSyncSettings | null,
  dependencies: MailActionDependencies
): Promise<void> {
  if (!isMailProvider(account.provider_kind)) return
  if (!current) {
    dependencies.setError(dependencies.t('Mail sync settings are not loaded yet.'))
    return
  }

  dependencies.setActiveAccount(account.account_id)
  dependencies.clearMessages()
  try {
    const normalized = normalizeMailSyncSettingsValues(current)
    await dependencies.updateMailSyncSettings({
      accountId: account.account_id,
      settings: {
        ...normalized,
        sync_enabled: enabled
      }
    })
    dependencies.setActionMessage(
      enabled ? dependencies.t('Mail service enabled') : dependencies.t('Mail service disabled')
    )
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : dependencies.t('Mail service update failed'))
  } finally {
    dependencies.setActiveAccount(null)
  }
}
