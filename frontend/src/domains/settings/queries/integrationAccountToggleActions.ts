import type { ConnectionProviderId } from '../../../shared/stores/integrationConnectionWizard'
import type { ProviderAccount } from '../types/settings'
import {
  defaultProviderIdFromAccount,
  isMailProvider
} from './integrationAccountPresentation'

type AccountToggleTranslator = (key: string) => string

interface AccountToggleDependencies {
  t: AccountToggleTranslator
  openConnectWizard: (providerId: ConnectionProviderId) => void
  logoutMailAccount: (accountId: string) => Promise<void>
  setError: (message: string) => void
}

export async function toggleSelectedAccount(
  account: ProviderAccount | null,
  enabled: boolean,
  dependencies: AccountToggleDependencies
): Promise<void> {
  if (!account) return

  if (!isMailProvider(account.provider_kind)) {
    dependencies.setError(dependencies.t('This provider does not expose a generic account toggle contract yet.'))
    return
  }

  if (enabled) {
    if (account.provider_kind !== 'gmail') {
      dependencies.setError(
        dependencies.t('This mail provider does not expose a self-serve reconnect flow in Settings yet.')
      )
      return
    }
    dependencies.openConnectWizard(defaultProviderIdFromAccount(account.provider_kind))
    return
  }

  await dependencies.logoutMailAccount(account.account_id)
}
